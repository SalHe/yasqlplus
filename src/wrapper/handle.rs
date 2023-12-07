use std::ptr::null_mut;

use crate::native::{
    yacAllocHandle, yacFreeHandle, EnYacHandleType_YAC_HANDLE_DBC, EnYacHandleType_YAC_HANDLE_ENV,
    EnYacHandleType_YAC_HANDLE_STMT, EnYacResult_YAC_ERROR, YacHandle,
};

use super::Error;

macro_rules! handle {
    ($name:ident => $handle_type:expr; $($input:ty)?) => {
        #[derive(Debug)]
        pub struct $name(pub YacHandle);

        impl $name {
            pub fn new($(input: &$input)?) -> Result<Self, Error> {
                let get_input = ||{
                    $(
                        return (input as &$input).0;
                        #[allow(unreachable_code)]
                    )?
                    null_mut()
                };
                let handle: YacHandle = null_mut();
                let result = unsafe {
                    yacAllocHandle(
                        $handle_type,
                        get_input(),
                        &handle as *const _ as *mut _,
                    )
                };
                if result != EnYacResult_YAC_ERROR {
                    Ok(Self(handle))
                } else {
                    Err(Error::get_yas_diag(None).unwrap())
                }
            }
        }

        impl Drop for $name {
            fn drop(&mut self) {
                unsafe {
                    yacFreeHandle($handle_type, self.0);
                }
            }
        }
    };
}

handle! {EnvHandle => EnYacHandleType_YAC_HANDLE_ENV;}
handle! {DbcHandle => EnYacHandleType_YAC_HANDLE_DBC; EnvHandle}
handle! {StatementHandle => EnYacHandleType_YAC_HANDLE_STMT; DbcHandle}
