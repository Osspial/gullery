use gullery::{
    geometry::{GLVec2, GLVec3, NonNormalized},
    image_format::{compressed::DXT1, ConcreteImageFormat, SRgb},
};
use png;

use std::{
    fs::File,
    io::{self, BufReader},
    path::{Path, PathBuf},
};

pub fn transform_path(path: &str) -> PathBuf {
    let mut path_transformed = Path::new(file!())
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf();
    path_transformed.push(path);
    path_transformed
}

#[allow(dead_code)]
pub fn load_dxt1_from_dds(path: &str) -> (Vec<Vec<DXT1<SRgb>>>, GLVec2<u32, NonNormalized>) {
    let path = transform_path(path);
    let mut file = BufReader::new(File::open(path).unwrap());
    let dds = ddsfile::Dds::read(&mut file).unwrap();

    let mut data = DXT1::<SRgb>::from_raw_slice(&dds.data);
    let mut mips = Vec::with_capacity(dds.header.mip_map_count.unwrap() as usize);
    println!("mip levels: {}", dds.header.mip_map_count.unwrap());
    println!("{:?}", data.len());
    for i in 0..dds.header.mip_map_count.unwrap() {
        let div = 2_u32.pow(i);
        let dims = GLVec3::new(dds.header.width / div, dds.header.height / div, 1);
        let split_index = DXT1::<SRgb>::blocks_for_dims(dims);
        println!("{:?} {:?} {}", i, dims, split_index);
        let mip = &data[..split_index];
        data = &data[split_index..];
        mips.push(mip.to_vec());
    }

    (mips, GLVec2::new(dds.header.width, dds.header.height))
}

#[allow(dead_code)]
pub fn load_png(path: &str) -> Result<(Vec<u8>, GLVec2<u32, NonNormalized>), io::Error> {
    let path = transform_path(path);
    let decoder = png::Decoder::new(File::open(path)?);
    let (info, mut reader) = decoder.read_info()?;
    let mut buf = vec![0; info.buffer_size()];
    reader.next_frame(&mut buf)?;
    Ok((buf, GLVec2::new(info.width, info.height)))
}
