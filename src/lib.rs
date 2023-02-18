use std::os::raw;
use std::ffi::{CString, CStr};
use std::ptr;
use std::mem;
use self::error::{NvrtcResult, ToResult};

mod nvrtc;
pub mod error;

pub fn version() -> NvrtcResult<(usize, usize)> {
    let mut major: raw::c_int = 0;
    let mut minor: raw::c_int = 0;
    unsafe {
        nvrtc::nvrtcVersion(&mut major, &mut minor).to_result()?;
    }
    Ok((major as usize, minor as usize))
}

#[derive(Debug)]
pub struct NvrtcProgram {
    inner: nvrtc::nvrtcProgram,
}

macro_rules! cstring_vec {
    ($vec:ident) => {{
        let cstr_vec: Vec<_> = $vec.iter().map(|s: &&str| CString::new(*s).unwrap()).collect();
        let mut pointer_vec: Vec<_> = cstr_vec.iter().map(|s| s.as_ptr()).collect();
        pointer_vec.push(ptr::null());
        (cstr_vec, pointer_vec)
    }.1 .as_ptr() as *const *const raw::c_char}; // cstr_vec cannot be dropped before pointer_vec
}

macro_rules! read_string_len {
    ($len:ident, $closure:expr) => {{
        let mut vec: Vec<u8> = Vec::with_capacity($len);
        unsafe {
            let read_fn: &dyn Fn(&mut Vec<u8>) -> Result<(), error::NvrtcError>  = &$closure;
            read_fn(&mut vec)?;
            // Ends with NULL byte
            vec.set_len($len-1);
        }
        Ok(String::from_utf8(vec)?)

    }}
}

macro_rules! read_vec_len {
    ($len:ident, $closure:expr) => {{
        let mut vec: Vec<u8> = Vec::with_capacity($len);
        unsafe {
            let read_fn: &dyn Fn(&mut Vec<u8>) -> Result<(), error::NvrtcError>  = &$closure;
            read_fn(&mut vec)?;
            // Ends with NULL byte
            vec.set_len($len);
        }
        Ok(vec)

    }}
}


impl NvrtcProgram {
    // headers done as tuple (source, name), so it will be more safe:
    // exact size of sources and names
    pub fn new(src: &str,
               name: Option<&str>,
               headers: &[&str],
               names: &[&str])
               -> NvrtcResult<Self> {
        let src_c = &CString::new(src).unwrap();
        let name = &CString::new(name.unwrap_or("default_program")).unwrap();

        debug_assert!(headers.len() == names.len(),
                      "NvrtcProgram::new: headers and names should be same");

        unsafe {
            let mut program = NvrtcProgram { inner: ptr::null_mut() };
            nvrtc::nvrtcCreateProgram(&mut program.inner as *mut nvrtc::nvrtcProgram,
                                      src_c.as_ptr() as *const raw::c_char,
                                      name.as_ptr() as *const raw::c_char,
                                      headers.len() as raw::c_int,
                                      cstring_vec!(headers),
                                      cstring_vec!(names)).to_result()?;
            Ok(program)
        }
    }

    pub fn compile(&self, opts: &[&str]) -> NvrtcResult<()> {
        unsafe {
            nvrtc::nvrtcCompileProgram(self.inner, opts.len() as raw::c_int, cstring_vec!(opts))
                .to_result()
        }
    }

    pub fn get_ptx_size(&self) -> NvrtcResult<usize> {
        let mut size: usize = 0;
        unsafe {
            nvrtc::nvrtcGetPTXSize(self.inner, &mut size).to_result()?;
        }
        Ok(size)
    }

    pub fn get_ptx(&self) -> Result<String, Box<dyn std::error::Error>> {
        let len = self.get_ptx_size()?;
        read_string_len!(len, |vec| {
            nvrtc::nvrtcGetPTX(self.inner, vec.as_mut_ptr() as *mut i8).to_result()
        })
    }

    pub fn get_cubin_size(&self) -> NvrtcResult<usize> {
        let mut size: usize = 0;
        unsafe {
            nvrtc::nvrtcGetCUBINSize(self.inner, &mut size).to_result()?;
        }
        Ok(size)
    }

    pub fn get_cubin(&self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let len = self.get_cubin_size()?;
        read_vec_len!(len, |vec| {
            nvrtc::nvrtcGetCUBIN(self.inner, vec.as_mut_ptr() as *mut i8).to_result()
        })
    }


    pub fn get_log_size(&self) -> NvrtcResult<usize> {
        let mut size: usize = 0;
        unsafe {
            nvrtc::nvrtcGetProgramLogSize(self.inner, &mut size).to_result()?;
        }
        Ok(size)
    }

    pub fn get_log(&self) -> Result<String, Box<dyn std::error::Error>> {
        let len = self.get_log_size()?;
        read_string_len!(len, |vec| {
            nvrtc::nvrtcGetProgramLog(self.inner, vec.as_mut_ptr() as *mut i8).to_result()
        })
    }

    pub fn add_expr(&self, expr: &str) -> NvrtcResult<()> {
        let expr_c = &CString::new(expr).unwrap();
        unsafe {
            nvrtc::nvrtcAddNameExpression(self.inner, expr_c.as_ptr() as *const raw::c_char)
                .to_result()
        }
    }
    pub fn get_name(&self, expr: &str) -> Result<String, Box<dyn std::error::Error>> {
        let expr_c = &CString::new(expr).unwrap();
        let mut ptr: *const raw::c_char = ptr::null();
        unsafe {
            nvrtc::nvrtcGetLoweredName(self.inner,
                                       expr_c.as_ptr() as *const raw::c_char,
                                       &mut ptr as *mut *const raw::c_char).to_result()?;
            let cstr = CStr::from_ptr(ptr).to_str()?;
            Ok(String::from(cstr))
        }
    }
}

impl Drop for NvrtcProgram {
    fn drop(&mut self) {
        if self.inner.is_null() {
            return;
        }
        unsafe {
            let mut inner = mem::replace(&mut self.inner, ptr::null_mut());
            nvrtc::nvrtcDestroyProgram(&mut inner)
                .to_result()
                .expect("Failed to destroy NVRTC program.");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // Output depends on compiler/cuda version, so we can test only if its won't crash
    #[test]
    fn nvrtc_version() {
        let (maj, min) = version().unwrap();
        println!("driver version = {}.{}", maj, min);
    }

    #[test]
    fn nvrtc_basic() {
        let src = r#"
            __global__ void say_hi2()
            {
                printf("Hi");
            }
        "#;
        let program = NvrtcProgram::new(src, Some("blah"), &[], &[]).unwrap();
        program.add_expr("say_hi2").unwrap();
        program.compile(&["-lineinfo", "-rdc=true"]).unwrap();
        println!("Log ({}):\n{}",
                 program.get_log_size().unwrap(),
                 program.get_log().unwrap());
        println!("PTX ({}):\n{}",
                 program.get_ptx_size().unwrap(),
                 program.get_ptx().unwrap());
        println!("New name:\n{}", program.get_name("say_hi2").unwrap());
    }

    #[test]
    fn nvrtc_headers() {
        let header = r#"
            __device__ void say_world()
            {
                printf("world\n");
            }
        "#;
        let src = r#"
            #include "world.cu"
            __global__ void say_hello()
            {
                printf("Hello");
                say_world();
            }
        "#;
        let program = NvrtcProgram::new(src, Some("/tmp/blah.cu"), &[header], &["world.cu"])
            .unwrap();
        program.add_expr("say_hello").unwrap();
        program.compile(&["-lineinfo"]).unwrap();

        let log = program.get_log().unwrap();
        println!("Log ({}):\n{}", program.get_log_size().unwrap(), log);

        let ptx = program.get_ptx().unwrap();
        println!("PTX ({}):\n{}", program.get_ptx_size().unwrap(), ptx);

        assert!(ptx.chars().last().unwrap()!='\0', "Last PTX byte is not NULL");
        
        println!("New name:\n{}", program.get_name("say_hello").unwrap());
    }
}
