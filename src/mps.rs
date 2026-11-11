//! Thin, typed wrapper over Apple's MPSGraph (Metal Performance Shaders Graph).
//!
//! This is the GPU engine on Apple Silicon: the whole network is compiled into
//! one Metal graph, which is what makes both inference and training fast here.
//! Everything is driven from Rust through `objc2` bindings; there is no C++,
//! Python or shader toolchain involved.
#![cfg(target_os = "macos")]

use std::ffi::c_void;
use std::ptr::NonNull;

use objc2::rc::{autoreleasepool, Retained};
use objc2::runtime::ProtocolObject;
use objc2::AnyThread;
use objc2_foundation::{NSArray, NSData, NSDictionary, NSNumber};
use objc2_metal::{MTLCommandQueue, MTLCreateSystemDefaultDevice, MTLDevice};
use objc2_metal_performance_shaders::MPSDataType;
use objc2_metal_performance_shaders_graph::{
    MPSGraph, MPSGraphDevice, MPSGraphOperation, MPSGraphTensor, MPSGraphTensorData,
};

pub type T = Retained<MPSGraphTensor>;
pub type Op = Retained<MPSGraphOperation>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Precision {
    F32,
    F16,
}

impl Precision {
    fn mps(self) -> MPSDataType {
        match self {
            Precision::F32 => MPSDataType::Float32,
            Precision::F16 => MPSDataType::Float16,
        }
    }
}

pub struct Graph {
    pub g: Retained<MPSGraph>,
    dev: Retained<MPSGraphDevice>,
    queue: Retained<ProtocolObject<dyn MTLCommandQueue>>,
    /// Precision used for parameters and activations.
    pub prec: Precision,
}

fn ints(v: &[isize]) -> Retained<NSArray<NSNumber>> {
    let n: Vec<Retained<NSNumber>> = v.iter().map(|&d| NSNumber::new_isize(d)).collect();
    NSArray::from_retained_slice(&n)
}

fn f32_bytes(v: &[f32]) -> &[u8] {
    unsafe { std::slice::from_raw_parts(v.as_ptr() as *const u8, std::mem::size_of_val(v)) }
}

impl Graph {
    pub fn new(prec: Precision) -> anyhow::Result<Self> {
        let mtl = MTLCreateSystemDefaultDevice().ok_or_else(|| anyhow::anyhow!("no Metal device"))?;
        let queue = mtl.newCommandQueue().ok_or_else(|| anyhow::anyhow!("no Metal command queue"))?;
        unsafe {
            Ok(Self { g: MPSGraph::new(), dev: MPSGraphDevice::deviceWithMTLDevice(&mtl), queue, prec })
        }
    }

    // ---- leaves -----------------------------------------------------------

    /// Float32 input placeholder (cast to the working precision with [`Graph::cast_in`]).
    pub fn placeholder(&self, shape: &[isize]) -> T {
        unsafe { self.g.placeholderWithShape_dataType_name(Some(&ints(shape)), MPSDataType::Float32, None) }
    }

    fn data_for(&self, data: &[f32]) -> Retained<NSData> {
        match self.prec {
            Precision::F32 => NSData::with_bytes(f32_bytes(data)),
            Precision::F16 => {
                let h: Vec<u16> = data.iter().map(|&x| half::f16::from_f32(x).to_bits()).collect();
                NSData::with_bytes(unsafe { std::slice::from_raw_parts(h.as_ptr() as *const u8, h.len() * 2) })
            }
        }
    }

    /// Frozen parameter.
    pub fn constant(&self, data: &[f32], shape: &[isize]) -> T {
        unsafe { self.g.constantWithData_shape_dataType(&self.data_for(data), &ints(shape), self.prec.mps()) }
    }

    /// Trainable (assignable) parameter.
    pub fn variable(&self, data: &[f32], shape: &[isize]) -> T {
        unsafe { self.g.variableWithData_shape_dataType_name(&self.data_for(data), &ints(shape), self.prec.mps(), None) }
    }

    pub fn scalar(&self, v: f64) -> T {
        unsafe { self.g.constantWithScalar_dataType(v, self.prec.mps()) }
    }

    pub fn cast_in(&self, x: &T) -> T {
        if self.prec == Precision::F32 {
            return x.clone();
        }
        unsafe { self.g.castTensor_toType_name(x, self.prec.mps(), None) }
    }

    pub fn cast_out(&self, x: &T) -> T {
        if self.prec == Precision::F32 {
            return x.clone();
        }
        unsafe { self.g.castTensor_toType_name(x, MPSDataType::Float32, None) }
    }

    // ---- ops --------------------------------------------------------------

    pub fn add(&self, a: &T, b: &T) -> T {
        unsafe { self.g.additionWithPrimaryTensor_secondaryTensor_name(a, b, None) }
    }
    pub fn sub(&self, a: &T, b: &T) -> T {
        unsafe { self.g.subtractionWithPrimaryTensor_secondaryTensor_name(a, b, None) }
    }
    pub fn mul(&self, a: &T, b: &T) -> T {
        unsafe { self.g.multiplicationWithPrimaryTensor_secondaryTensor_name(a, b, None) }
    }
    pub fn div(&self, a: &T, b: &T) -> T {
        unsafe { self.g.divisionWithPrimaryTensor_secondaryTensor_name(a, b, None) }
    }
    pub fn max(&self, a: &T, b: &T) -> T {
        unsafe { self.g.maximumWithPrimaryTensor_secondaryTensor_name(a, b, None) }
    }
    pub fn scale(&self, a: &T, s: f64) -> T {
        self.mul(a, &self.scalar(s))
    }
    pub fn matmul(&self, a: &T, b: &T) -> T {
        unsafe { self.g.matrixMultiplicationWithPrimaryTensor_secondaryTensor_name(a, b, None) }
    }
    pub fn reshape(&self, x: &T, shape: &[isize]) -> T {
        unsafe { self.g.reshapeTensor_withShape_name(x, &ints(shape), None) }
    }
    pub fn transpose(&self, x: &T, a: usize, b: usize) -> T {
        unsafe { self.g.transposeTensor_dimension_withDimension_name(x, a, b, None) }
    }
    pub fn permute(&self, x: &T, perm: &[isize]) -> T {
        unsafe { self.g.transposeTensor_permutation_name(x, &ints(perm), None) }
    }
    pub fn concat(&self, xs: &[T], dim: isize) -> T {
        unsafe { self.g.concatTensors_dimension_name(&NSArray::from_retained_slice(xs), dim, None) }
    }
    pub fn slice(&self, x: &T, dim: usize, start: isize, len: isize) -> T {
        unsafe { self.g.sliceTensor_dimension_start_length_name(x, dim, start, len, None) }
    }
    pub fn softmax(&self, x: &T, axis: isize) -> T {
        unsafe { self.g.softMaxWithTensor_axis_name(x, axis, None) }
    }
    pub fn erf(&self, x: &T) -> T {
        unsafe { self.g.erfWithTensor_name(x, None) }
    }
    pub fn exp(&self, x: &T) -> T {
        unsafe { self.g.exponentWithTensor_name(x, None) }
    }
    pub fn ln(&self, x: &T) -> T {
        unsafe { self.g.logarithmWithTensor_name(x, None) }
    }
    pub fn abs(&self, x: &T) -> T {
        unsafe { self.g.absoluteWithTensor_name(x, None) }
    }
    pub fn sqrt(&self, x: &T) -> T {
        unsafe { self.g.squareRootWithTensor_name(x, None) }
    }
    pub fn square(&self, x: &T) -> T {
        unsafe { self.g.squareWithTensor_name(x, None) }
    }
    pub fn mean(&self, x: &T, axes: &[isize]) -> T {
        unsafe { self.g.meanOfTensor_axes_name(x, &ints(axes), None) }
    }
    pub fn sum(&self, x: &T, axes: &[isize]) -> T {
        unsafe { self.g.reductionSumWithTensor_axes_name(x, Some(&ints(axes)), None) }
    }

    /// Exact (erf) GELU, matching timm / PyTorch `nn.GELU()`.
    pub fn gelu(&self, x: &T) -> T {
        let e = self.erf(&self.scale(x, std::f64::consts::FRAC_1_SQRT_2));
        self.mul(&self.scale(x, 0.5), &self.add(&e, &self.scalar(1.0)))
    }

    /// Layer normalisation over the last axis of a rank-`rank` tensor.
    pub fn layer_norm(&self, x: &T, gamma: &T, beta: &T, eps: f32, rank: isize) -> T {
        let axes = ints(&[rank - 1]);
        unsafe {
            let mean = self.g.meanOfTensor_axes_name(x, &axes, None);
            let var = self.g.varianceOfTensor_axes_name(x, &axes, None);
            self.g.normalizationWithTensor_meanTensor_varianceTensor_gammaTensor_betaTensor_epsilon_name(
                x, &mean, &var, Some(gamma), Some(beta), eps, None,
            )
        }
    }

    // ---- autodiff / state ---------------------------------------------------

    /// d(loss)/d(param) for every parameter, in the same order.
    pub fn gradients(&self, loss: &T, params: &[T]) -> Vec<T> {
        unsafe {
            let d = self.g.gradientForPrimaryTensor_withTensors_name(loss, &NSArray::from_retained_slice(params), None);
            params.iter().map(|p| d.objectForKey(p).expect("missing gradient")).collect()
        }
    }

    pub fn assign(&self, var: &T, value: &T) -> Op {
        unsafe { self.g.assignVariable_withValueOfTensor_name(var, value, None) }
    }

    pub fn read(&self, var: &T) -> T {
        unsafe { self.g.readVariable_name(var, None) }
    }

    // ---- execution ----------------------------------------------------------

    /// Run the graph. `feeds` are `(placeholder, data, shape)`; returns one
    /// `Vec<f32>` per target (targets must be Float32).
    pub fn run(&self, feeds: &[(&T, &[f32], &[isize])], targets: &[T], ops: &[Op]) -> Vec<Vec<f32>> {
        autoreleasepool(|_| unsafe {
            let keys: Vec<&MPSGraphTensor> = feeds.iter().map(|f| &**f.0).collect();
            let vals: Vec<Retained<MPSGraphTensorData>> = feeds
                .iter()
                .map(|(_, data, shape)| {
                    MPSGraphTensorData::initWithDevice_data_shape_dataType(
                        MPSGraphTensorData::alloc(),
                        &self.dev,
                        &NSData::with_bytes(f32_bytes(data)),
                        &ints(shape),
                        MPSDataType::Float32,
                    )
                })
                .collect();
            let feeds = NSDictionary::from_retained_objects(&keys, &vals);
            let tgt = NSArray::from_retained_slice(targets);
            let ops_arr = NSArray::from_retained_slice(ops);
            let out = self.g.runWithMTLCommandQueue_feeds_targetTensors_targetOperations(
                &self.queue,
                &feeds,
                &tgt,
                if ops.is_empty() { None } else { Some(&ops_arr) },
            );
            targets
                .iter()
                .map(|t| {
                    let td = out.objectForKey(t).expect("missing target");
                    let n: usize = td.shape().iter().map(|d| d.as_usize()).product();
                    let mut buf = vec![0f32; n];
                    td.mpsndarray().readBytes_strideBytes(
                        NonNull::new(buf.as_mut_ptr() as *mut c_void).unwrap(),
                        std::ptr::null_mut(),
                    );
                    buf
                })
                .collect()
        })
    }
}
