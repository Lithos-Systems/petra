#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

use crate::{blocks::Block, signal::SignalBus, error::{Result, PlcError}, value::Value};

/// SIMD-optimized array math operations
pub struct SimdArrayAdd {
    name: String,
    input_a: String,
    input_b: String,
    output: String,
}

impl SimdArrayAdd {
    pub fn new(name: String, input_a: String, input_b: String, output: String) -> Self {
        Self { name, input_a, input_b, output }
    }

    #[cfg(target_arch = "x86_64")]
    #[target_feature(enable = "avx2")]
    unsafe fn add_arrays_avx2(a: &[f64], b: &[f64]) -> Vec<f64> {
        let len = a.len().min(b.len());
        let mut result = Vec::with_capacity(len);

        // Process 4 elements at a time with AVX2
        let chunks = len / 4;
        for i in 0..chunks {
            let idx = i * 4;
            let a_vec = _mm256_loadu_pd(a.as_ptr().add(idx));
            let b_vec = _mm256_loadu_pd(b.as_ptr().add(idx));
            let sum = _mm256_add_pd(a_vec, b_vec);

            let mut temp = [0.0; 4];
            _mm256_storeu_pd(temp.as_mut_ptr(), sum);
            result.extend_from_slice(&temp);
        }

        // Handle remaining elements
        for i in (chunks * 4)..len {
            result.push(a[i] + b[i]);
        }

        result
    }
}

impl Block for SimdArrayAdd {
    fn execute(&mut self, bus: &SignalBus) -> Result<()> {
        let a = bus.get(&self.input_a)?;
        let b = bus.get(&self.input_b)?;

        match (a, b) {
            #[cfg(feature = "extended-types")]
            (Value::Array(arr_a), Value::Array(arr_b)) => {
                // Extract float arrays
                let floats_a: Result<Vec<f64>> = arr_a.iter()
                    .map(|v| match v {
                        Value::Float(f) => Ok(*f),
                        _ => Err(PlcError::Type("Array must contain floats".into()))
                    })
                    .collect();

                let floats_b: Result<Vec<f64>> = arr_b.iter()
                    .map(|v| match v {
                        Value::Float(f) => Ok(*f),
                        _ => Err(PlcError::Type("Array must contain floats".into()))
                    })
                    .collect();

                let a_floats = floats_a?;
                let b_floats = floats_b?;

                #[cfg(target_arch = "x86_64")]
                let result = if is_x86_feature_detected!("avx2") {
                    unsafe { Self::add_arrays_avx2(&a_floats, &b_floats) }
                } else {
                    a_floats.iter().zip(b_floats.iter())
                        .map(|(a, b)| a + b)
                        .collect()
                };

                #[cfg(not(target_arch = "x86_64"))]
                let result: Vec<f64> = a_floats.iter().zip(b_floats.iter())
                    .map(|(a, b)| a + b)
                    .collect();

                let result_array = result.into_iter()
                    .map(Value::Float)
                    .collect();

                bus.set(&self.output, Value::Array(result_array))?;
            }
            _ => {
                return Err(PlcError::Type("SIMD array operations require array inputs".into()));
            }
        }

        Ok(())
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn block_type(&self) -> &str {
        "SIMD_ARRAY_ADD"
    }
}
