pub trait ToAstring {
    fn to_astring(self) -> String;
}

impl ToAstring for i64 {
    fn to_astring(self) -> String {
        itoa::Buffer::new().format(self).to_owned()
    }
}

impl ToAstring for f64 {
    fn to_astring(self) -> String {
        ryu::Buffer::new().format(self).to_owned()
    }
}
