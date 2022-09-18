use std::fs;

use crate::arg::InteractiveMode;
use crate::core::{is_write_protected, RmStatus};
use crate::interact;

#[must_use]
pub fn prompt(metadata: &fs::Metadata, name: &str, mode: InteractiveMode) -> RmStatus {
    let write_protected = is_write_protected(metadata);
    let empty = metadata.len() == 0;

    let message = format!(
        "rm: remove{write_protected}regular{empty}file '{name}'?",
        write_protected = if write_protected {
            " write-protected "
        } else {
            " "
        },
        empty = if empty { " empty " } else { " " },
        name = name
    );

    let maybe_interact;
    match mode {
        InteractiveMode::Always => {
            maybe_interact = interact::with_message(message);
        }
        InteractiveMode::Once | InteractiveMode::Never => {
            if write_protected {
                maybe_interact = interact::with_message(message);
            } else {
                return RmStatus::Accept;
            }
        }
    }

    if let Ok(yes) = maybe_interact {
        if yes {
            return RmStatus::Accept;
        }
        return RmStatus::Declined;
    }

    RmStatus::Failed(maybe_interact.unwrap_err())
}
