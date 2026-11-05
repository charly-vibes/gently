//! Purpose: autosplit node parts and port resolution — record/HTML-like
//! label splitting and the autosplit node lookup an edge port names.
//! Responsibilities: split record labels on bars and HTML labels on TD
//! cells, track the parts per base node, and resolve `base:port` ports
//! onto part indices.
//! Rationale: split from dot/mod.rs past the tidy file limit.

use super::{err, record, ParseError, P};
use crate::graph::Scope;

impl P<'_> {
    pub(super) fn resolve_port(
        &mut self,
        base: &str,
        port: &str,
        compass: Option<&str>,
    ) -> Result<usize, ParseError> {
        let port_err = |p: &P| {
            err(
                p.line(),
                format!(
                    "Cannot find autosplit node for {base}:{port} on edge {}",
                    p.g.edges.len() + 1
                ),
            )
        };
        let parts = match self.split.get(base) {
            Some(parts) => parts.clone(),
            None => return Err(port_err(self)),
        };
        if let Ok(i) = port.parse::<usize>() {
            return parts.get(i).copied().ok_or_else(|| port_err(self));
        }
        if super::autosplit::is_compass_point(port, compass) {
            Ok(parts[0])
        } else {
            Err(port_err(self))
        }
    }
    pub(super) fn apply_node_attrs(
        &mut self,
        idx: usize,
        list: &[(String, String)],
    ) -> Result<(), ParseError> {
        for (k, v) in list {
            self.g.set_attr(Scope::Node(idx), k, v);
        }
        self.maybe_autosplit(idx)
    }
    fn maybe_autosplit(&mut self, idx: usize) -> Result<(), ParseError> {
        let shape = self
            .g
            .get_attr(Scope::Node(idx), "shape")
            .unwrap_or("")
            .to_string();
        let label = match self.g.get_attr(Scope::Node(idx), "label") {
            Some(label) => label.to_string(),
            None => return Ok(()),
        };
        if shape == "record" {
            if label.contains('{') || label.contains('}') {
                let escaped = record::escape_record_braces(&label);
                self.g.set_attr(Scope::Node(idx), "label", &escaped);
                self.split_record(idx, &escaped)
            } else if label.contains('|') {
                self.split_record(idx, &label)
            } else {
                Ok(())
            }
        } else if label.starts_with('<') && label.ends_with('>') && label.len() >= 2 {
            self.split_html(idx, &label)
        } else {
            Ok(())
        }
    }
    fn split_record(&mut self, idx: usize, label: &str) -> Result<(), ParseError> {
        let base = self.g.nodes[idx].name.clone();
        self.g.nodes[idx].name = label.to_string();
        self.g.nodes[idx].attributes.remove("label");
        self.g.set_attr(Scope::Node(idx), "basename", &base);
        self.make_parts(base, label.split('|').count());
        Ok(())
    }
    fn split_html(&mut self, idx: usize, label: &str) -> Result<(), ParseError> {
        let texts = match record::html_parts(&label[1..label.len() - 1]) {
            Ok(texts) if !texts.is_empty() => texts,
            _ => {
                return Err(err(
                    self.line(),
                    format!(
                        "'{}' not recognized by Graph::Easy::Parser::Graphviz",
                        label
                    ),
                ))
            }
        };
        let base = self.g.nodes[idx].name.clone();
        self.g.nodes[idx].name = label.to_string();
        self.g.nodes[idx].attributes.remove("label");
        self.make_parts(base, texts.len());
        Ok(())
    }
    fn make_parts(&mut self, base: String, count: usize) {
        let parts: Vec<usize> = (0..count)
            .map(|i| self.g.add_node(&format!("{}.{}", base, i)))
            .collect();
        self.split.insert(base, parts);
    }
}

/// Is `port` a compass point — a named compass spelling, or any token
/// when the port argument was already typed as compass?
pub(super) fn is_compass_point(port: &str, compass: Option<&str>) -> bool {
    compass.is_some()
        || matches!(
            port,
            "n" | "ne" | "e" | "se" | "s" | "sw" | "w" | "nw" | "c"
        )
}
