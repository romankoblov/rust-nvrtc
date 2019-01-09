use std::error::Error;
use std::ffi::CStr;
use std::fmt;
use std::mem;
use super::nvrtc;
use super::nvrtc::nvrtcResult as nvrtcResult_t;

#[repr(u32)]
#[allow(missing_docs)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NvrtcError {
    OutOfMemory = 1,
    ProgramCreationFailure = 2,
    InvalidInput = 3,
    InvalidProgram = 4,
    InvalidOption = 5,
    Compilation = 6,
    BuiltinOperationFailure = 7,
    NoNameExpressionsAfterCompilation = 8,
    NoLoweredNamesBeforeCompilation = 9,
    NameExpressionNotValid = 10,
    InternalError = 11,
    UnknownError = 999,
}

impl fmt::Display for NvrtcError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            other if (other as u32) <= 999 => {
                let value = other as u32;
                unsafe {
                    let cstr = CStr::from_ptr(nvrtc::nvrtcGetErrorString(mem::transmute(value)));
                    write!(f, "{:?}", cstr)
                }
            }
            _ => write!(f, "Unknown error"),
        }
    }
}

impl Error for NvrtcError {}

pub type NvrtcResult<T> = Result<T, NvrtcError>;

pub(crate) trait ToResult {
    fn to_result(self) -> NvrtcResult<()>;
}

#[allow(unreachable_patterns)]
impl ToResult for nvrtcResult_t {
    fn to_result(self) -> NvrtcResult<()> {
        match self {
            nvrtcResult_t::NVRTC_SUCCESS => Ok(()),
            nvrtcResult_t::NVRTC_ERROR_OUT_OF_MEMORY => Err(NvrtcError::OutOfMemory),
            nvrtcResult_t::NVRTC_ERROR_PROGRAM_CREATION_FAILURE => {
                Err(NvrtcError::ProgramCreationFailure)
            }
            nvrtcResult_t::NVRTC_ERROR_INVALID_INPUT => Err(NvrtcError::InvalidInput),
            nvrtcResult_t::NVRTC_ERROR_INVALID_PROGRAM => Err(NvrtcError::InvalidProgram),
            nvrtcResult_t::NVRTC_ERROR_INVALID_OPTION => Err(NvrtcError::InvalidOption),
            nvrtcResult_t::NVRTC_ERROR_COMPILATION => Err(NvrtcError::Compilation),
            nvrtcResult_t::NVRTC_ERROR_BUILTIN_OPERATION_FAILURE => {
                Err(NvrtcError::BuiltinOperationFailure)
            }
            nvrtcResult_t::NVRTC_ERROR_NO_NAME_EXPRESSIONS_AFTER_COMPILATION => {
                Err(NvrtcError::NoNameExpressionsAfterCompilation)
            }
            nvrtcResult_t::NVRTC_ERROR_NO_LOWERED_NAMES_BEFORE_COMPILATION => {
                Err(NvrtcError::NoLoweredNamesBeforeCompilation)
            }
            nvrtcResult_t::NVRTC_ERROR_NAME_EXPRESSION_NOT_VALID => {
                Err(NvrtcError::NameExpressionNotValid)
            }
            nvrtcResult_t::NVRTC_ERROR_INTERNAL_ERROR => Err(NvrtcError::InternalError),
            _ => Err(NvrtcError::UnknownError),
        }
    }
}
