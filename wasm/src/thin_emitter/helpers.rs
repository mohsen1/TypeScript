use super::ThinPrinter;
use crate::source_writer::SourcePosition;

impl<'a> ThinPrinter<'a> {
    fn take_pending_source_pos(&mut self) -> Option<SourcePosition> {
        self.pending_source_pos.take()
    }

    // =========================================================================
    // Output Helpers (delegate to SourceWriter)
    // pub(super) for access from submodules (expressions, statements, declarations)
    // =========================================================================

    /// Write text to output.
    pub(super) fn write(&mut self, text: &str) {
        if let Some(source_pos) = self.take_pending_source_pos() {
            self.writer.write_node(text, source_pos);
        } else {
            self.writer.write(text);
        }
    }

    /// Write identifier text to output with name mapping when available.
    pub(super) fn write_identifier(&mut self, text: &str) {
        if let Some(source_pos) = self.take_pending_source_pos() {
            self.writer.write_node_with_name(text, source_pos, text);
        } else {
            self.writer.write(text);
        }
    }

    /// Write a single character.
    pub(super) fn write_char(&mut self, ch: char) {
        if let Some(source_pos) = self.take_pending_source_pos() {
            let mut buf = [0u8; 4];
            let text = ch.encode_utf8(&mut buf);
            self.writer.write_node(text, source_pos);
        } else {
            self.writer.write_char(ch);
        }
    }

    /// Write a newline.
    pub(super) fn write_line(&mut self) {
        self.writer.write_line();
    }

    /// Write a space.
    pub(super) fn write_space(&mut self) {
        self.writer.write_space();
    }

    /// Write an unsigned integer.
    pub(super) fn write_usize(&mut self, value: usize) {
        if let Some(source_pos) = self.take_pending_source_pos() {
            if value == 0 {
                self.writer.write_node("0", source_pos);
                return;
            }

            let mut buf = [0u8; 20];
            let mut i = buf.len();
            let mut remaining = value;
            while remaining > 0 {
                let digit = (remaining % 10) as u8;
                i -= 1;
                buf[i] = b'0' + digit;
                remaining /= 10;
            }

            // SAFETY: buffer only contains ASCII digits.
            let digits = unsafe { std::str::from_utf8_unchecked(&buf[i..]) };
            self.writer.write_node(digits, source_pos);
        } else {
            self.writer.write_usize(value);
        }
    }

    /// Write a semicolon (respecting options).
    pub(super) fn write_semicolon(&mut self) {
        if !self.ctx.options.omit_trailing_semicolon {
            self.write(";");
        }
    }

    /// Increase indentation.
    pub(super) fn increase_indent(&mut self) {
        self.writer.increase_indent();
    }

    /// Decrease indentation.
    pub(super) fn decrease_indent(&mut self) {
        self.writer.decrease_indent();
    }
}
