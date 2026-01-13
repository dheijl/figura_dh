use crate::{Context, Value};
use std::borrow::Cow;
use std::fmt::{self, Write};

pub trait Directive {
    fn format(&self, ctx: &Context, buf: &mut String) -> fmt::Result;
}

pub struct EmptyDirective;

impl Directive for EmptyDirective {
    fn format(&self, _ctx: &Context, _buf: &mut String) -> fmt::Result {
        Ok(())
    }
}

pub struct LiteralDirective<'a>(pub Cow<'a, str>);

impl<'a> Directive for LiteralDirective<'a> {
    fn format(&self, _ctx: &Context, buf: &mut String) -> fmt::Result {
        buf.push_str(&self.0);
        Ok(())
    }
}

pub struct ReplaceDirective<'a>(pub &'a str);

impl<'a> Directive for ReplaceDirective<'a> {
    fn format(&self, ctx: &Context, buf: &mut String) -> fmt::Result {
        if let Some(val) = ctx.get(self.0) {
            val.write_to(buf)?;
        }

        Ok(())
    }
}

pub struct RepeatDirective<'a>(pub Cow<'a, str>, pub &'a str);

impl<'a> Directive for RepeatDirective<'a> {
    fn format(&self, ctx: &Context, buf: &mut String) -> fmt::Result {
        let pattern = ctx
            .get(&self.0.as_ref())
            .map(|v| v.as_cow())
            .unwrap_or(Cow::Borrowed(&self.0));

        match ctx.get(&self.1) {
            Some(count) => match count {
                Value::Int(i) => write!(buf, "{}", pattern.repeat(*i as usize))?,
                _ => {}
            },

            None => {
                if let Ok(count) = self.1.parse::<usize>() {
                    write!(buf, "{}", pattern.repeat(count))?;
                }
            }
        }

        Ok(())
    }
}
