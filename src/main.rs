use std::ffi::c_float;

mod cuda;
use cuda::*;

fn main() {
    let n = 10;

    let (mut a, mut b, mut c) = (vec![0f32; n], vec![0f32; n], vec![0f32; n]);

    for i in 0..n {
        a[i] = i as c_float;
        b[i] = 2.0 * i as c_float;
    }

    let size = n * size_of::<c_float>();

    let (mut cuda_a, mut cuda_b, mut cuda_c) =
        (CudaMem::new(size).unwrap(), CudaMem::new(size).unwrap(), CudaMem::new(size).unwrap());

    unsafe {
        cuda_a.copy_in(&a).unwrap();
        cuda_b.copy_in(&b).unwrap();

        kernels::add_vector(&cuda_a, &cuda_b, &mut cuda_c, n).unwrap();

        cuda_c.copy_out(&mut c).unwrap();
    };

    for i in 0..n {
        println!("{} + {} = {}", a[i], b[i], c[i]);
    }
}
