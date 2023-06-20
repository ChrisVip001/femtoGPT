use super::*;

pub fn gpu_run(out_id: TensorId, inps: &[Vec<usize>]) -> GpuFunction {
    let works = inps[0][..inps[0].len() - 1].iter().fold(1, |a, b| a * b);
    let group_size = inps[0].last().unwrap();
    let cnt = inps.len();
    let mut args = String::new();
    let mut code = String::new();
    for g in 0..cnt {
        args += &format!(",__global float* a{g}");
        code +=
            &format!("out[(id * {cnt} + {g}) * {group_size} + i] = a{g}[id * {group_size} + i];\n");
    }
    let source_code = format!(
        "__kernel void calc_{out_id}(
                        __global float* out
                        {args}) {{
        uint id = get_global_id(0);
        if(id < {works}) {{
            for(uint i = 0; i < {group_size}; i++) {{
                {code}
            }}
        }}
    }}"
    );

    let local_work_size = 32;
    let global_work_size =
        works + ((local_work_size - (works % local_work_size)) % local_work_size);

    GpuFunction {
        source_code,
        kernel_name: format!("calc_{}", out_id),
        local_work_size,
        global_work_size,
    }
}

pub fn gpu_grad(out_id: TensorId, inps: &[Vec<usize>]) -> GpuFunctionGroup {
    let works = inps[0][..inps[0].len() - 1].iter().fold(1, |a, b| a * b);
    let group_size = inps[0].last().unwrap();
    let cnt = inps.len();
    let mut args = String::new();
    let mut code = String::new();
    for g in 0..cnt {
        args += &format!(",__global float* a{g},__global float* a{g}_grad");
        code += &format!(
            "a{g}_grad[id * {group_size} + i] += out_grad[(id * {cnt} + {g}) * {group_size} + i];\n"
        );
    }

    let source_code = format!(
        "__kernel void grad_{out_id}(
                        __global float* out,
                        __global float* out_grad
                        {args}) {{
        uint id = get_global_id(0);
        if(id < {works}) {{
            for(uint i = 0; i < {group_size}; i++) {{
                {code}
            }}
        }}
    }}"
    );

    let local_work_size = 32;
    let global_work_size =
        works + ((local_work_size - (works % local_work_size)) % local_work_size);

    GpuFunctionGroup {
        funcs: vec![GpuFunction {
            source_code,
            kernel_name: format!("grad_{}", out_id),
            local_work_size,
            global_work_size,
        }],
        shared_buffers: vec![],
    }
}