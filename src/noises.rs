
use noise::{NoiseFn, OpenSimplex, Perlin, PerlinSurflet, Simplex, SuperSimplex, Value, Worley, Fbm, Billow, BasicMulti, RidgedMulti, HybridMulti};
use std::slice::Iter;
use bevy::prelude::Vec3;

#[derive(Clone, Debug)]
pub struct Noise {
    pub typ: NoiseType,
    pub seed: u32,
    pub scale: f64,
    pub octaves: usize,
    pub freq: f64,
    pub global: bool
}
impl Noise {
    pub fn new() -> Self {
        Noise { 
            typ:        NoiseType::Perlin, 
            seed:       1, 
            scale:      1.0, 
            octaves:    6, 
            freq:       1.0,
            global:     false
        }
    }
    fn _set(&self) -> NoiseFunction {
        let nfn = NoiseFunction::new(self.typ.clone(), self.seed, self.octaves, self.freq);
        return nfn;
    }
    pub fn apply(&self, loc: Vec3) -> f32 {
        let noise = self._set();
        let r = noise.apply(self.scale, loc.x as f64, loc.z as f64);
        return r as f32;
    }
}





#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NoiseType {
    Perlin,
    PerlinSurflet,
    OpenSimplex,
    Value,
    SuperSimplex,
    Worley,
    Simplex,
    FBMPerlin, // Fractal
    BMPerlin, //Basic Multi
    BPerlin,
    RMPerlin, // RidgedMultiPerlin
    HMPerlin, //Hybrid Multi perlin
    FBMPerlinSurflet,
    BMPerlinSurflet,
    BPerlinSurflet,
    RMPerlinSurflet,
    HMPerlinSurflet,
    FBMValue,
    BMValue,
    BValue,
    RMValue,
    HMValue,
    FBMSS,
    BMSS,
    BSS,
    RMSS,
    HMSS 

}

#[derive(Clone)]
enum NoiseFunction {
    Perlin(Perlin),
    PerlinSurflet(PerlinSurflet),
    OpenSimplex(OpenSimplex),
    SuperSimplex(SuperSimplex),
    Simplex(Simplex),
    Value(Value),
    Worley(Worley),
    BMPerlin(BasicMulti<Perlin>),
    FBMPerlin(Fbm<Perlin>),
    BPerlin(Billow<Perlin>),
    RMPerlin(RidgedMulti<Perlin>),
    HMPerlin(HybridMulti<Perlin>),
    BMPerlinSurflet(BasicMulti<PerlinSurflet>),
    FBMPerlinSurflet(Fbm<PerlinSurflet>),
    BPerlinSurflet(Billow<PerlinSurflet>),
    RMPerlinSurflet(RidgedMulti<PerlinSurflet>),
    HMPerlinSurflet(HybridMulti<PerlinSurflet>),
    BMValue(BasicMulti<Value>),
    FBMValue(Fbm<Value>),
    BValue(Billow<Value>),
    RMValue(RidgedMulti<Value>),
    HMValue(HybridMulti<Value>),
    BMSS(BasicMulti<SuperSimplex>),
    FBMSS(Fbm<SuperSimplex>),
    BSS(Billow<SuperSimplex>),
    RMSS(RidgedMulti<SuperSimplex>),
    HMSS(HybridMulti<SuperSimplex>)
}

impl NoiseFunction {
    fn apply(&self, scale: f64, x: f64, z: f64) -> f64 {
        let r: f64;
        match &self {
            // XD but I really dont know how to make it better
            NoiseFunction::Perlin(f)           => {r = f.get([x* scale, z * scale])}
            NoiseFunction::PerlinSurflet(f)    => {r = f.get([x* scale, z * scale])}
            NoiseFunction::Value(f)            => {r = f.get([x* scale, z * scale])}
            NoiseFunction::OpenSimplex(f)      => {r = f.get([x* scale, z * scale])}
            NoiseFunction::SuperSimplex(f)     => {r = f.get([x* scale, z * scale])}
            NoiseFunction::Worley(f)           => {r = f.get([x* scale, z * scale])}
            NoiseFunction::Simplex(f)          => {r = f.get([x* scale, z * scale])}
            NoiseFunction::FBMPerlin(f)        => {r = f.get([x* scale, z * scale])}
            NoiseFunction::BMPerlin(f)         => {r = f.get([x* scale, z * scale])}
            NoiseFunction::BPerlin(f)          => {r = f.get([x* scale, z * scale])}
            NoiseFunction::RMPerlin(f)         => {r = f.get([x* scale, z * scale])}
            NoiseFunction::HMPerlin(f)         => {r = f.get([x* scale, z * scale])}
            NoiseFunction::FBMPerlinSurflet(f) => {r = f.get([x* scale, z * scale])}
            NoiseFunction::BMPerlinSurflet(f)  => {r = f.get([x* scale, z * scale])}
            NoiseFunction::BPerlinSurflet(f)   => {r = f.get([x* scale, z * scale])}
            NoiseFunction::RMPerlinSurflet(f)  => {r = f.get([x* scale, z * scale])}
            NoiseFunction::HMPerlinSurflet(f)  => {r = f.get([x* scale, z * scale])}
            NoiseFunction::FBMValue(f)         => {r = f.get([x* scale, z * scale])}
            NoiseFunction::BMValue(f)          => {r = f.get([x* scale, z * scale])}
            NoiseFunction::BValue(f)           => {r = f.get([x* scale, z * scale])}
            NoiseFunction::RMValue(f)          => {r = f.get([x* scale, z * scale])}
            NoiseFunction::HMValue(f)          => {r = f.get([x* scale, z * scale])}
            NoiseFunction::FBMSS(f)            => {r = f.get([x* scale, z * scale])}
            NoiseFunction::BMSS(f)             => {r = f.get([x* scale, z * scale])}
            NoiseFunction::BSS(f)              => {r = f.get([x* scale, z * scale])}
            NoiseFunction::RMSS(f)             => {r = f.get([x* scale, z * scale])}
            NoiseFunction::HMSS(f)             => {r = f.get([x* scale, z * scale])}
        }
        return r;
    }

    fn _apply3d(&self, scale: f64, x: f64, y: f64, z: f64) -> f64 {
        let r: f64;
        match &self {
            // XD but I really dont know how to make it better
            NoiseFunction::Perlin(f)           => {r = f.get([x* scale, y*scale, z * scale])}
            NoiseFunction::PerlinSurflet(f)    => {r = f.get([x* scale, y*scale, z * scale])}
            NoiseFunction::Value(f)            => {r = f.get([x* scale, y*scale, z * scale])}
            NoiseFunction::OpenSimplex(f)      => {r = f.get([x* scale, y*scale, z * scale])}
            NoiseFunction::SuperSimplex(f)     => {r = f.get([x* scale, y*scale, z * scale])}
            NoiseFunction::Worley(f)           => {r = f.get([x* scale, y*scale, z * scale])}
            NoiseFunction::Simplex(f)          => {r = f.get([x* scale, y*scale, z * scale])}
            NoiseFunction::FBMPerlin(f)        => {r = f.get([x* scale, y*scale, z * scale])}
            NoiseFunction::BMPerlin(f)         => {r = f.get([x* scale, y*scale, z * scale])}
            NoiseFunction::BPerlin(f)          => {r = f.get([x* scale, y*scale, z * scale])}
            NoiseFunction::RMPerlin(f)         => {r = f.get([x* scale, y*scale, z * scale])}
            NoiseFunction::HMPerlin(f)         => {r = f.get([x* scale, y*scale, z * scale])}
            NoiseFunction::FBMPerlinSurflet(f) => {r = f.get([x* scale, y*scale, z * scale])}
            NoiseFunction::BMPerlinSurflet(f)  => {r = f.get([x* scale, y*scale, z * scale])}
            NoiseFunction::BPerlinSurflet(f)   => {r = f.get([x* scale, y*scale, z * scale])}
            NoiseFunction::RMPerlinSurflet(f)  => {r = f.get([x* scale, y*scale, z * scale])}
            NoiseFunction::HMPerlinSurflet(f)  => {r = f.get([x* scale, y*scale, z * scale])}
            NoiseFunction::FBMValue(f)         => {r = f.get([x* scale, y*scale, z * scale])}
            NoiseFunction::BMValue(f)          => {r = f.get([x* scale, y*scale, z * scale])}
            NoiseFunction::BValue(f)           => {r = f.get([x* scale, y*scale, z * scale])}
            NoiseFunction::RMValue(f)          => {r = f.get([x* scale, y*scale, z * scale])}
            NoiseFunction::HMValue(f)          => {r = f.get([x* scale, y*scale, z * scale])}
            NoiseFunction::FBMSS(f)            => {r = f.get([x* scale, y*scale, z * scale])}
            NoiseFunction::BMSS(f)             => {r = f.get([x* scale, y*scale, z * scale])}
            NoiseFunction::BSS(f)              => {r = f.get([x* scale, y*scale, z * scale])}
            NoiseFunction::RMSS(f)             => {r = f.get([x* scale, y*scale, z * scale])}
            NoiseFunction::HMSS(f)             => {r = f.get([x* scale, y*scale, z * scale])}
        }
        return r;
    }

 

    fn new(noise: NoiseType, seed: u32, octaves: usize, freq: f64) -> Self {
        let nfn: NoiseFunction;
        match noise {
            NoiseType::Perlin =>        {nfn = NoiseFunction::Perlin(Perlin::new(seed))}
            NoiseType::PerlinSurflet => {nfn = NoiseFunction::PerlinSurflet(PerlinSurflet::new(seed))}
            NoiseType::Value =>         {nfn = NoiseFunction::Value(Value::new(seed))}
            NoiseType::OpenSimplex =>   {nfn = NoiseFunction::OpenSimplex(OpenSimplex::new(seed))}
            NoiseType::SuperSimplex =>  {nfn = NoiseFunction::SuperSimplex(SuperSimplex::new(seed))}
            NoiseType::Worley =>        {nfn = NoiseFunction::Worley(Worley::new(seed))}
            NoiseType::Simplex =>       {nfn = NoiseFunction::Simplex(Simplex::new(seed))}
            NoiseType::FBMPerlin =>     {
                let mut noise_fn: Fbm<Perlin> = Fbm::new(seed);
                noise_fn.octaves = octaves;
                noise_fn.frequency = freq;
                nfn = NoiseFunction::FBMPerlin(noise_fn);
            }
            NoiseType::BMPerlin => {
                let mut noise_fn: BasicMulti<Perlin> = BasicMulti::new(seed);
                noise_fn.octaves = octaves;
                noise_fn.frequency = freq;
                nfn = NoiseFunction::BMPerlin(noise_fn);
            }
            NoiseType::BPerlin => {
                let mut noise_fn: Billow<Perlin> = Billow::new(seed);
                noise_fn.octaves = octaves;
                noise_fn.frequency = freq;
                nfn = NoiseFunction::BPerlin(noise_fn);
            }
            NoiseType::RMPerlin => {
                let mut noise_fn: RidgedMulti<Perlin> = RidgedMulti::new(seed);
                noise_fn.octaves = octaves;
                noise_fn.frequency = freq;
                nfn = NoiseFunction::RMPerlin(noise_fn);
            }
            NoiseType::HMPerlin => {
                let mut noise_fn: HybridMulti<Perlin> = HybridMulti::new(seed);
                noise_fn.octaves = octaves;
                noise_fn.frequency = freq;
                nfn =  NoiseFunction::HMPerlin(noise_fn);
            }
            NoiseType::FBMPerlinSurflet =>     {
                let mut noise_fn: Fbm<PerlinSurflet> = Fbm::new(seed);
                noise_fn.octaves = octaves;
                noise_fn.frequency = freq;
                nfn =  NoiseFunction::FBMPerlinSurflet(noise_fn);
            }
            NoiseType::BMPerlinSurflet => {
                let mut noise_fn: BasicMulti<PerlinSurflet> = BasicMulti::new(seed);
                noise_fn.octaves = octaves;
                noise_fn.frequency = freq;
                nfn =  NoiseFunction::BMPerlinSurflet(noise_fn);
            }
            NoiseType::BPerlinSurflet => {
                let mut noise_fn: Billow<PerlinSurflet> = Billow::new(seed);
                noise_fn.octaves = octaves;
                noise_fn.frequency = freq;
                nfn =  NoiseFunction::BPerlinSurflet(noise_fn);
            }
            NoiseType::RMPerlinSurflet => {
                let mut noise_fn: RidgedMulti<PerlinSurflet> = RidgedMulti::new(seed);
                noise_fn.octaves = octaves;
                noise_fn.frequency = freq;
                nfn =  NoiseFunction::RMPerlinSurflet(noise_fn);
            }
            NoiseType::HMPerlinSurflet => {
                let mut noise_fn: HybridMulti<PerlinSurflet> = HybridMulti::new(seed);
                noise_fn.octaves = octaves;
                noise_fn.frequency = freq;
                nfn =  NoiseFunction::HMPerlinSurflet(noise_fn);
            }
            NoiseType::FBMValue =>     {
                let mut noise_fn: Fbm<Value> = Fbm::new(seed);
                noise_fn.octaves = octaves;
                noise_fn.frequency = freq;
                nfn =  NoiseFunction::FBMValue(noise_fn);
            }
            NoiseType::BMValue => {
                let mut noise_fn: BasicMulti<Value> = BasicMulti::new(seed);
                noise_fn.octaves = octaves;
                noise_fn.frequency = freq;
                nfn =  NoiseFunction::BMValue(noise_fn);
            }
            NoiseType::BValue => {
                let mut noise_fn: Billow<Value> = Billow::new(seed);
                noise_fn.octaves = octaves;
                noise_fn.frequency = freq;
                nfn =  NoiseFunction::BValue(noise_fn);
            }
            NoiseType::RMValue => {
                let mut noise_fn: RidgedMulti<Value> = RidgedMulti::new(seed);
                noise_fn.octaves = octaves;
                noise_fn.frequency = freq;
                nfn =  NoiseFunction::RMValue(noise_fn);
            }
            NoiseType::HMValue => {
                let mut noise_fn: HybridMulti<Value> = HybridMulti::new(seed);
                noise_fn.octaves = octaves;
                noise_fn.frequency = freq;
                nfn =  NoiseFunction::HMValue(noise_fn);
            }
            NoiseType::FBMSS =>     {
                let mut noise_fn: Fbm<SuperSimplex> = Fbm::new(seed);
                noise_fn.octaves = octaves;
                noise_fn.frequency = freq;
                nfn =  NoiseFunction::FBMSS(noise_fn);
            }
            NoiseType::BMSS => {
                let mut noise_fn: BasicMulti<SuperSimplex> = BasicMulti::new(seed);
                noise_fn.octaves = octaves;
                noise_fn.frequency = freq;
                nfn =  NoiseFunction::BMSS(noise_fn);
            }
            NoiseType::BSS => {
                let mut noise_fn: Billow<SuperSimplex> = Billow::new(seed);
                noise_fn.octaves = octaves;
                noise_fn.frequency = freq;
                nfn =  NoiseFunction::BSS(noise_fn);
            }
            NoiseType::RMSS => {
                let mut noise_fn: RidgedMulti<SuperSimplex> = RidgedMulti::new(seed);
                noise_fn.octaves = octaves;
                noise_fn.frequency = freq;
                nfn =  NoiseFunction::RMSS(noise_fn);
            }
            NoiseType::HMSS => {
                let mut noise_fn: HybridMulti<SuperSimplex> = HybridMulti::new(seed);
                noise_fn.octaves = octaves;
                noise_fn.frequency = freq;
                nfn = NoiseFunction::HMSS(noise_fn);
            }
            
        }
        return nfn;
    }
}



impl<'a> NoiseType {
    pub fn iterator() -> Iter<'static, NoiseType> {
    static NOISES_OPTIONS: [NoiseType; 27] = [
        NoiseType::Perlin,
        NoiseType::PerlinSurflet,
        NoiseType::OpenSimplex,
        NoiseType::Value,
        NoiseType::SuperSimplex,
        NoiseType::Worley,
        NoiseType::Simplex,
        NoiseType::FBMPerlin,
        NoiseType::BMPerlin,
        NoiseType::BPerlin,
        NoiseType::RMPerlin,
        NoiseType::HMPerlin,
        NoiseType::FBMPerlinSurflet,
        NoiseType::BMPerlinSurflet,
        NoiseType::BPerlinSurflet,
        NoiseType::RMPerlinSurflet,
        NoiseType::HMPerlinSurflet,
        NoiseType::FBMValue,
        NoiseType::BMValue,
        NoiseType::BValue,
        NoiseType::RMValue,
        NoiseType::HMValue,
        NoiseType::FBMSS,
        NoiseType::BMSS,
        NoiseType::BSS,
        NoiseType::RMSS,
        NoiseType::HMSS 
    ];
    NOISES_OPTIONS.iter()
  }
}