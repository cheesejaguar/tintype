//! The pieces of training a shallower student that do not need the GPU: which
//! of a teacher's blocks a student starts from, and the target a student is
//! trained towards when a teacher has a say in it. `train` puts them
//! together (`--blocks`, `--init-blocks`, `--distill`); see
//! docs/runbooks/cpu-profile-int8.md.
use crate::weights::Weights;
use anyhow::{ensure, Context, Result};

/// How many blocks a checkpoint has (`blocks.0`, `blocks.1`, ... without a gap).
pub fn depth_of(w: &Weights) -> usize {
    (0..).take_while(|i| w.0.contains_key(&format!("blocks.{i}.norm1.weight"))).count()
}

/// `0,1,2,3,5,7,9,11`: the teacher's blocks a student takes, in order.
pub fn parse_blocks(list: &str, teacher_depth: usize) -> Result<Vec<usize>> {
    let picks: Vec<usize> = list.split(',').map(|s| s.trim().parse::<usize>().with_context(|| format!("`{s}` is not a block number"))).collect::<Result<_>>()?;
    ensure!(!picks.is_empty() && picks.windows(2).all(|p| p[0] < p[1]), "blocks are listed once each, in increasing order: `{list}`");
    ensure!(picks.iter().all(|&p| p < teacher_depth), "the checkpoint has blocks 0 to {}: `{list}`", teacher_depth.saturating_sub(1));
    Ok(picks)
}

/// A checkpoint whose block `i` is block `picks[i]` of `w`; every other
/// tensor (embeddings, final normalisation, head) is copied as it is. The
/// blocks left out are gone.
pub fn take_blocks(w: &Weights, picks: &[usize]) -> Result<Weights> {
    let depth = depth_of(w);
    ensure!(picks.iter().all(|&p| p < depth), "the checkpoint has {depth} blocks");
    let mut out = Weights::default();
    for (name, (data, shape)) in &w.0 {
        match name.strip_prefix("blocks.").and_then(|rest| rest.split_once('.')) {
            Some((index, tensor)) => {
                let from: usize = index.parse().with_context(|| format!("tensor `{name}`"))?;
                if let Some(to) = picks.iter().position(|&p| p == from) {
                    out.insert(&format!("blocks.{to}.{tensor}"), data.clone(), shape.clone());
                }
            }
            None => out.insert(name, data.clone(), shape.clone()),
        }
    }
    Ok(out)
}

/// The target of the binary cross-entropy when a share `weight` of it is the
/// teacher's: `(1 - weight) * label + weight * sigmoid(teacher_logit /
/// temperature)`. Cross-entropy is linear in its target, so training towards
/// this is `(1 - weight)` of the loss against the label plus `weight` of the
/// loss against the teacher's softened probability. The trainer builds the
/// same expression in its graph.
pub fn soft_target(label: f32, teacher_logit: f32, weight: f32, temperature: f32) -> f32 {
    (1.0 - weight) * label + weight * crate::metrics::sigmoid(teacher_logit / temperature)
}

/// Binary cross-entropy of a logit against a target in `[0, 1]`, in the
/// numerically stable form the trainer uses.
pub fn bce_with_logits(logit: f32, target: f32) -> f32 {
    logit.max(0.0) - logit * target + (1.0 + (-logit.abs()).exp()).ln()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn checkpoint(depth: usize) -> Weights {
        let mut w = Weights::default();
        w.insert("cls_token", vec![0.5; 4], vec![1, 1, 4]);
        w.insert("norm.weight", vec![1.0; 4], vec![4]);
        w.insert("head.weight", vec![0.25; 4], vec![1, 4]);
        for i in 0..depth {
            w.insert(&format!("blocks.{i}.norm1.weight"), vec![i as f32; 4], vec![4]);
            w.insert(&format!("blocks.{i}.attn.qkv.weight"), vec![10.0 + i as f32; 48], vec![12, 4]);
        }
        w
    }

    #[test]
    fn a_student_takes_the_teachers_blocks_it_names_and_everything_outside_them() {
        let teacher = checkpoint(12);
        assert_eq!(depth_of(&teacher), 12);
        let picks = parse_blocks("0,1,2,3,5,7,9,11", 12).unwrap();
        assert_eq!(picks, [0, 1, 2, 3, 5, 7, 9, 11]);
        let student = take_blocks(&teacher, &picks).unwrap();
        assert_eq!(depth_of(&student), 8);
        for (to, &from) in picks.iter().enumerate() {
            assert_eq!(student.get(&format!("blocks.{to}.norm1.weight")).unwrap().0, vec![from as f32; 4]);
            assert_eq!(student.get(&format!("blocks.{to}.attn.qkv.weight")).unwrap(), &(vec![10.0 + from as f32; 48], vec![12, 4]));
        }
        assert!(student.get("blocks.8.norm1.weight").is_err());
        for name in ["cls_token", "norm.weight", "head.weight"] {
            assert_eq!(student.get(name).unwrap(), teacher.get(name).unwrap());
        }
        // Tensor counts: three outside the blocks and two per block kept.
        assert_eq!(student.0.len(), 3 + 2 * 8);
        // The first blocks, as `--blocks` alone takes them.
        assert_eq!(depth_of(&take_blocks(&teacher, &[0, 1, 2, 3, 4, 5]).unwrap()), 6);
        assert!(take_blocks(&teacher, &[0, 12]).is_err());
    }

    #[test]
    fn block_lists_are_increasing_and_inside_the_teacher() {
        assert_eq!(parse_blocks("0, 2 ,4", 6).unwrap(), [0, 2, 4]);
        for bad in ["", "0,0", "2,1", "0,6", "a", "0,,1", "-1"] {
            assert!(parse_blocks(bad, 6).is_err(), "`{bad}`");
        }
    }

    #[test]
    fn the_soft_target_mixes_the_label_with_the_teachers_softened_probability() {
        // No weight: the label. All weight: the teacher's probability.
        assert_eq!(soft_target(1.0, -3.0, 0.0, 1.0), 1.0);
        assert!((soft_target(1.0, 0.0, 1.0, 1.0) - 0.5).abs() < 1e-7);
        // Half and half, a confident teacher that disagrees with the label.
        let t = soft_target(0.0, 4.0, 0.5, 1.0);
        assert!((t - 0.5 * (1.0 / (1.0 + (-4.0f32).exp()))).abs() < 1e-6);
        // A higher temperature softens the teacher towards a half.
        assert!(soft_target(0.0, 4.0, 1.0, 4.0) < soft_target(0.0, 4.0, 1.0, 1.0) && soft_target(0.0, 4.0, 1.0, 4.0) > 0.5);
        // The loss against the mix is the mix of the losses.
        for (z, y, zt) in [(0.3f32, 1.0f32, 2.0f32), (-1.5, 0.0, -0.5), (4.0, 0.0, 6.0)] {
            let p = crate::metrics::sigmoid(zt / 2.0);
            let mixed = bce_with_logits(z, soft_target(y, zt, 0.3, 2.0));
            assert!((mixed - (0.7 * bce_with_logits(z, y) + 0.3 * bce_with_logits(z, p))).abs() < 1e-5);
        }
        // The stable form is the textbook one.
        let (z, y) = (0.7f32, 0.2f32);
        let p = crate::metrics::sigmoid(z);
        assert!((bce_with_logits(z, y) - (-(y * p.ln() + (1.0 - y) * (1.0 - p).ln()))).abs() < 1e-6);
    }
}
