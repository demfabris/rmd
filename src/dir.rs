use std::ffi::OsStr;
use std::{fs, path};

use crate::arg::{InteractiveMode, RmOptions};
use crate::core::{is_write_protected, RmStatus};
use crate::error::Error;
use crate::interact;

#[must_use]
/// # Panics
pub fn prompt<'a>(
    opt: &RmOptions,
    path: &'a OsStr,
    metadata: &fs::Metadata,
    name: &str,
    mode: InteractiveMode,
    visited: bool,
) -> RmStatus<'a> {
    let is_empty_dir = path::PathBuf::from(path)
        .read_dir()
        .unwrap()
        .next()
        .is_none();

    if !opt.recursive {
        if !opt.dir {
            return RmStatus::Failed(Error::IsDirectory(name.to_owned()));
        }

        if opt.dir && !is_empty_dir {
            return RmStatus::Failed(Error::DirectoryNotEmpty(name.to_owned()));
        }
    }

    let write_protected = is_write_protected(metadata);
    let descend = opt.recursive && !is_empty_dir && !visited;
    let message = format!(
        "rm: {descend_remove}{write_protected}directory '{name}'?",
        descend_remove = if descend { "descend into" } else { "remove" },
        write_protected = if write_protected {
            " write-protected "
        } else {
            " "
        },
        name = name
    );

    let mut force_accept = false;
    let maybe_interact = match mode {
        InteractiveMode::Always => {
            if (is_empty_dir && opt.dir) || opt.recursive {
                interact::with_message(message)
            } else {
                force_accept = true;
                Ok(false)
            }
        }
        InteractiveMode::Once => {
            force_accept = true;
            Ok(false)
        }
        InteractiveMode::Never => {
            if write_protected {
                interact::with_message(message)
            } else {
                force_accept = true;
                Ok(false)
            }
        }
    };

    if force_accept {
        return RmStatus::Accept;
    }

    if let Ok(yes) = maybe_interact {
        if yes && descend {
            return RmStatus::Descend(path);
        } else if yes {
            return RmStatus::Accept;
        }
        return RmStatus::Declined;
    }

    RmStatus::Failed(maybe_interact.unwrap_err())
}
