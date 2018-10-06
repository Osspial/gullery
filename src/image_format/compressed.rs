#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BC4<S> {
    pub red: [S; 8]
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BC5<S> {
    pub red: [S; 8],
    pub green: [S; 8]
}
