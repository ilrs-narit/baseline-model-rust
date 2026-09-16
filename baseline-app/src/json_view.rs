//! Helpers for the "JSON" tab in each mode: it renders a preview of the
//! decoded Data Table rows as JSON and offers a "Download JSON..." button that writes every row to a file.

use serde_json::Value;
use std::path::PathBuf;

pub const PREVIEW_ROWS: usize = 20;

/// Parses `s` into a JSON number when it looks like one, otherwise keeps it as
/// a string (hex markers like `E225`, timestamps, reserved/checksum spans).
pub fn num_or_str(s: &str) -> Value {
    let t = s.trim();
    if let Ok(i) = t.parse::<i64>() {
        return Value::from(i);
    }
    if let Ok(f) = t.parse::<f64>() {
        if f.is_finite() {
            return Value::from(f);
        }
    }
    Value::from(t)
}

/// Builds an ordered JSON object from `(key, value)` pairs, keeping insertion
/// order (serde_json's `preserve_order` feature is enabled).
pub fn object<I>(pairs: I) -> Value
where
    I: IntoIterator<Item = (String, Value)>,
{
    Value::Object(pairs.into_iter().collect())
}

/// True when every element of `arr` is a scalar (not an object/array), so the
/// array can be printed on one line, e.g. `[1, 2, 3, 4, 5]`.
fn is_flat_array(arr: &[Value]) -> bool {
    arr.iter().all(|v| !matches!(v, Value::Array(_) | Value::Object(_)))
}

fn write_indent(out: &mut String, depth: usize) {
    for _ in 0..depth {
        out.push_str("  ");
    }
}

/// Writes `value` as pretty JSON into `out`, 2-space indented like
/// `serde_json::to_string_pretty`, except that scalar arrays (numbers,
/// strings, bools, null - no nested objects/arrays) are kept on one line.
fn write_pretty(value: &Value, depth: usize, out: &mut String) {
    match value {
        Value::Array(arr) if arr.is_empty() => out.push_str("[]"),
        Value::Array(arr) if is_flat_array(arr) => {
            out.push('[');
            for (i, item) in arr.iter().enumerate() {
                if i > 0 {
                    out.push_str(", ");
                }
                write_pretty(item, depth, out);
            }
            out.push(']');
        }
        Value::Array(arr) => {
            out.push_str("[\n");
            let inner = depth + 1;
            for (i, item) in arr.iter().enumerate() {
                write_indent(out, inner);
                write_pretty(item, inner, out);
                if i + 1 < arr.len() {
                    out.push(',');
                }
                out.push('\n');
            }
            write_indent(out, depth);
            out.push(']');
        }
        Value::Object(map) if map.is_empty() => out.push_str("{}"),
        Value::Object(map) => {
            out.push_str("{\n");
            let inner = depth + 1;
            let len = map.len();
            for (i, (key, v)) in map.iter().enumerate() {
                write_indent(out, inner);
                out.push_str(&serde_json::to_string(key).unwrap_or_default());
                out.push_str(": ");
                write_pretty(v, inner, out);
                if i + 1 < len {
                    out.push(',');
                }
                out.push('\n');
            }
            write_indent(out, depth);
            out.push('}');
        }
        scalar => out.push_str(&serde_json::to_string(scalar).unwrap_or_default()),
    }
}

/// Pretty-prints `value`, keeping scalar arrays (e.g. the 16 decoded values
/// of a block) on one line instead of one element per line.
pub fn to_pretty_string(value: &Value) -> String {
    let mut out = String::new();
    write_pretty(value, 0, &mut out);
    out
}

/// Pretty-prints the first [`PREVIEW_ROWS`] of `values` for the on-screen
/// preview. Returns an empty string when there is nothing to show.
pub fn preview_string(values: &[Value]) -> String {
    if values.is_empty() {
        return String::new();
    }
    let shown = &values[..values.len().min(PREVIEW_ROWS)];
    to_pretty_string(&Value::Array(shown.to_vec()))
}

/// Renders the JSON tab body. Returns `true` when the download button was
/// clicked this frame.
pub fn json_tab_ui(ui: &mut egui::Ui, row_count: usize, preview_json: &str) -> bool {
    let mut download_clicked = false;
    ui.horizontal(|ui| {
        if ui
            .add_enabled(row_count > 0, egui::Button::new("Download JSON..."))
            .clicked()
        {
            download_clicked = true;
        }
        ui.label(format!("{row_count} row(s)"));
        if row_count > PREVIEW_ROWS {
            ui.separator();
            ui.label(format!(
                "Preview shows the first {PREVIEW_ROWS}"
            ));
        }
    });
    ui.separator();

    if preview_json.is_empty() {
        ui.label("No data - run Process Data first.");
        return download_clicked;
    }

    egui::ScrollArea::both()
        .id_salt("json_preview_scroll")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.add(
                egui::Label::new(egui::RichText::new(preview_json).monospace())
                    .selectable(true)
                    .extend(),
            );
        });
    download_clicked
}

/// Prompts for a path and writes `values` as pretty JSON. `Ok(None)` means the
/// user cancelled the dialog.
pub fn save_json(default_name: &str, values: &[Value]) -> Result<Option<PathBuf>, String> {
    let Some(path) = rfd::FileDialog::new()
        .set_file_name(default_name)
        .add_filter("JSON", &["json"])
        .save_file()
    else {
        return Ok(None);
    };
    let text = to_pretty_string(&Value::Array(values.to_vec()));
    std::fs::write(&path, text).map_err(|e| e.to_string())?;
    Ok(Some(path))
}

