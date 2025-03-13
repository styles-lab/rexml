use std::io::{Result, Write};

use crate::types::XmlVersion;

/// A low-level xml document writer without semnatic check.
pub struct XmlWriter<W>
where
    W: Write,
{
    /// underlying write.
    sink: W,
}

impl<W> XmlWriter<W>
where
    W: Write,
{
    /// Create a xml document writer from [`std::io::Write`].
    pub fn new(sink: W) -> Self {
        Self { sink }
    }

    pub fn write_xml_decl(
        &mut self,
        version: XmlVersion,
        encoding: Option<&str>,
        standalone: Option<bool>,
    ) -> Result<()> {
        self.sink
            .write_fmt(format_args!("<?xml version={}", version))?;

        if let Some(encoding) = encoding {
            self.sink
                .write_fmt(format_args!(" encoding={}", encoding))?;
        }

        if let Some(standalone) = standalone {
            self.sink.write_fmt(format_args!(
                " standalone={}",
                if standalone { "yes" } else { "no" }
            ))?;
        }

        self.sink.write_all(b"?>")?;

        Ok(())
    }

    /// Write pi node.
    pub fn write_pi<N, U>(&mut self, name: N, unparsed: U) -> Result<()>
    where
        N: AsRef<str>,
        U: AsRef<str>,
    {
        self.sink
            .write_fmt(format_args!("<?{} {} ?>", name.as_ref(), unparsed.as_ref()))?;

        Ok(())
    }

    /// Write comment node.
    pub fn write_comment<C>(&mut self, content: C) -> Result<()>
    where
        C: AsRef<str>,
    {
        self.sink
            .write_fmt(format_args!("<!--{}-->", content.as_ref()))?;

        Ok(())
    }

    /// Write cdata.
    pub fn write_cdata<C>(&mut self, content: C) -> Result<()>
    where
        C: AsRef<str>,
    {
        self.sink
            .write_fmt(format_args!("<![CDATA[{}]]>", content.as_ref()))?;

        Ok(())
    }

    /// Write cdata.
    pub fn write_chardata<C>(&mut self, content: C) -> Result<()>
    where
        C: AsRef<str>,
    {
        self.sink.write_all(content.as_ref().as_bytes())?;

        Ok(())
    }

    /// Start write element start tag.
    pub fn write_elment_start<N>(&mut self, name: N) -> Result<ElemStartWrite<'_, W>>
    where
        N: AsRef<str>,
    {
        self.sink.write_fmt(format_args!("<{}", name.as_ref()))?;

        Ok(ElemStartWrite {
            sink: self,
            is_empty: false,
        })
    }

    /// Start write empty element start tag.
    pub fn write_empty_elment<N>(&mut self, name: N) -> Result<ElemStartWrite<'_, W>>
    where
        N: AsRef<str>,
    {
        self.sink.write_fmt(format_args!("<{}", name.as_ref()))?;

        Ok(ElemStartWrite {
            sink: self,
            is_empty: true,
        })
    }

    /// Write a element end tag.
    pub fn write_element_end(&mut self, name: &str) -> Result<()> {
        self.sink.write_fmt(format_args!("</{}>", name))?;

        Ok(())
    }
}

impl<W> Drop for XmlWriter<W>
where
    W: Write,
{
    fn drop(&mut self) {
        if let Err(err) = self.sink.flush() {
            log::error!("{}", err);
        }
    }
}

/// A write for element start tag.
pub struct ElemStartWrite<'a, W>
where
    W: Write,
{
    sink: &'a mut XmlWriter<W>,
    is_empty: bool,
}

impl<'a, W> Drop for ElemStartWrite<'a, W>
where
    W: Write,
{
    fn drop(&mut self) {
        if let Err(err) = if self.is_empty {
            self.sink.sink.write_all(b"/>")
        } else {
            self.sink.sink.write_all(b">")
        } {
            log::error!("{}", err);
        }
    }
}

impl<'a, W> ElemStartWrite<'a, W>
where
    W: Write,
{
    /// Write new attribute value pair.
    pub fn write_attr<N, V>(&mut self, name: N, value: V) -> Result<()>
    where
        N: AsRef<str>,
        V: AsRef<str>,
    {
        if value.as_ref().contains('"') {
            self.sink
                .sink
                .write_fmt(format_args!(" {}='{}'", name.as_ref(), value.as_ref()))
        } else {
            self.sink
                .sink
                .write_fmt(format_args!(" {}=\"{}\"", name.as_ref(), value.as_ref()))
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::types::XmlVersion;

    use super::XmlWriter;

    #[test]
    fn test_write() {
        let mut writer = XmlWriter::new(Vec::new());

        writer
            .write_xml_decl(XmlVersion::Ver11, None, None)
            .unwrap();

        writer.write_comment("helloworld").unwrap();

        let mut el = writer.write_elment_start("svg").unwrap();

        el.write_attr("hello", "world").unwrap();

        drop(el);

        writer.write_chardata("\nhello world").unwrap();

        writer.write_element_end("svg").unwrap();
    }
}
