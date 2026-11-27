"""One timed process for a released AIGIBench Setting-II detector (PROTOCOL-DDA.md, addendum 4): the repository's own test
options, data loader and network (HorizonTEL/AIGIBench detector_codes at 70d5b75e, checkpoints at 6c423c5b), FP32 on MPS.
python -I aigib_speed_aigibcode.py <repo dir> <checkpoint> <ImageFolder root with 0_real/ and 1_fake/>
The test settings are AIGIBench's own (test.py: no_resize False, no_crop True); batch 32 and 8 loader workers, as for DDA and
DGS-Net. Prints images scored and flagged (sigmoid > 0.5), nothing per file."""
import os, sys

NET = {'NPR-DeepfakeDetection-main': ('networks.resnet', 'resnet50'), 'FreqNet-DeepfakeDetection-main': ('networks.freqnet', 'freqnet'),
       'DFFreq-main': ('networks.resnet', 'resnet50'), 'BSF-main': ('networks.resnet', 'resnet50'), 'Resnet50-main': ('networks.resnet', 'resnet50')}


def main():
    repo, ckpt, root = sys.argv[1], sys.argv[2], sys.argv[3]
    os.chdir(repo); sys.path.insert(0, repo); sys.argv = ['test.py', '--gpu_ids', '-1']
    import importlib, torch, torch.multiprocessing as tmp
    # The repositories' transforms hold local lambdas, which only Linux-style forked workers can carry, so the loader's
    # workers are forked, and before the model touches the Apple GPU. `.cuda()` calls in the released code go to MPS.
    tmp.set_start_method('fork', force=True)
    torch.Tensor.cuda = lambda self, *a, **k: self.to('mps')
    torch.nn.Module.cuda = lambda self, *a, **k: self.to('mps')
    from options.test_options import TestOptions
    from data import create_dataloader
    opt = TestOptions().parse(print_options=False)
    opt.dataroot, opt.classes, opt.no_resize, opt.no_crop = root, '', False, True
    opt.batch_size, opt.num_threads, opt.serial_batches = 32, 8, True
    it = iter(create_dataloader(opt))  # forks the 8 workers now
    mod, fn = NET[os.path.basename(os.path.normpath(repo))]
    model = getattr(importlib.import_module(mod), fn)(num_classes=1)
    model.load_state_dict(torch.load(ckpt, map_location='cpu'), strict=True)
    model = model.to('mps').eval()
    n = fl = 0
    with torch.inference_mode():
        for batch in it:
            x = batch[0]
            out = model(x.to('mps')); out = out[0] if isinstance(out, (tuple, list)) else out
            p = out.sigmoid().flatten().float().cpu(); n += len(p); fl += int((p > 0.5).sum())
    print(f'{os.path.basename(os.path.normpath(repo))}: scored {n}, flagged {fl}')


if __name__ == '__main__':
    main()
