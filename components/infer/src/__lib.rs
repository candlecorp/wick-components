#![feature(prelude_import)]
#![feature(prelude_import)]
#[prelude_import]
use std::prelude::rust_2021::*;
#[macro_use]
extern crate std;
#[prelude_import]
use std::prelude::rust_2021::*;
#[macro_use]
extern crate std;
mod inference {
    mod model {
        use candle_core::{DType, Device, IndexOp, Result, Tensor, D};
        use candle_nn::linear_no_bias as linear;
        use candle_nn::{embedding, rms_norm, Embedding, Linear, Module, RmsNorm, VarBuilder};
        use std::collections::HashMap;
        use std::sync::{Arc, Mutex};
        pub(crate) struct Config {
            pub(crate) dim: usize,
            pub(crate) hidden_dim: usize,
            pub(crate) n_layers: usize,
            pub(crate) n_heads: usize,
            pub(crate) n_kv_heads: usize,
            pub(crate) vocab_size: usize,
            pub(crate) seq_len: usize,
            pub(crate) norm_eps: f64,
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for Config {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                let names: &'static _ = &[
                    "dim",
                    "hidden_dim",
                    "n_layers",
                    "n_heads",
                    "n_kv_heads",
                    "vocab_size",
                    "seq_len",
                    "norm_eps",
                ];
                let values: &[&dyn ::core::fmt::Debug] = &[
                    &self.dim,
                    &self.hidden_dim,
                    &self.n_layers,
                    &self.n_heads,
                    &self.n_kv_heads,
                    &self.vocab_size,
                    &self.seq_len,
                    &&self.norm_eps,
                ];
                ::core::fmt::Formatter::debug_struct_fields_finish(f, "Config", names, values)
            }
        }
        #[automatically_derived]
        impl ::core::clone::Clone for Config {
            #[inline]
            fn clone(&self) -> Config {
                Config {
                    dim: ::core::clone::Clone::clone(&self.dim),
                    hidden_dim: ::core::clone::Clone::clone(&self.hidden_dim),
                    n_layers: ::core::clone::Clone::clone(&self.n_layers),
                    n_heads: ::core::clone::Clone::clone(&self.n_heads),
                    n_kv_heads: ::core::clone::Clone::clone(&self.n_kv_heads),
                    vocab_size: ::core::clone::Clone::clone(&self.vocab_size),
                    seq_len: ::core::clone::Clone::clone(&self.seq_len),
                    norm_eps: ::core::clone::Clone::clone(&self.norm_eps),
                }
            }
        }
        impl Config {
            pub(crate) fn _tiny() -> Self {
                Self {
                    dim: 288,
                    hidden_dim: 768,
                    n_layers: 6,
                    n_heads: 6,
                    n_kv_heads: 6,
                    vocab_size: 32000,
                    seq_len: 256,
                    norm_eps: 1e-5,
                }
            }
        }
        pub(crate) struct Cache {
            masks: Arc<Mutex<HashMap<usize, Tensor>>>,
            pub(crate) use_kv_cache: bool,
            #[allow(clippy::type_complexity)]
            kvs: Arc<Mutex<Vec<Option<(Tensor, Tensor)>>>>,
            cos: Tensor,
            sin: Tensor,
            device: Device,
        }
        #[automatically_derived]
        impl ::core::clone::Clone for Cache {
            #[inline]
            fn clone(&self) -> Cache {
                Cache {
                    masks: ::core::clone::Clone::clone(&self.masks),
                    use_kv_cache: ::core::clone::Clone::clone(&self.use_kv_cache),
                    kvs: ::core::clone::Clone::clone(&self.kvs),
                    cos: ::core::clone::Clone::clone(&self.cos),
                    sin: ::core::clone::Clone::clone(&self.sin),
                    device: ::core::clone::Clone::clone(&self.device),
                }
            }
        }
        impl Cache {
            pub(crate) fn new(use_kv_cache: bool, cfg: &Config, vb: VarBuilder) -> Result<Self> {
                let n_elem = cfg.dim / cfg.n_heads;
                let theta: Vec<_> = (0..n_elem)
                    .step_by(2)
                    .map(|i| 1f32 / 10000f32.powf(i as f32 / n_elem as f32))
                    .collect();
                let theta = Tensor::new(theta.as_slice(), vb.device())?;
                let idx_theta = Tensor::arange(0, cfg.seq_len as u32, vb.device())?
                    .to_dtype(DType::F32)?
                    .reshape((cfg.seq_len, 1))?
                    .matmul(&theta.reshape((1, theta.elem_count()))?)?;
                let precomputed_cos = idx_theta.cos()?;
                let precomputed_sin = idx_theta.sin()?;
                let freq_cis_real = vb
                    .get((cfg.seq_len, cfg.head_size() / 2), "freq_cis_real")
                    .unwrap_or(precomputed_cos);
                let freq_cis_imag = vb
                    .get((cfg.seq_len, cfg.head_size() / 2), "freq_cis_imag")
                    .unwrap_or(precomputed_sin);
                let cos = freq_cis_real.reshape((cfg.seq_len, cfg.head_size() / 2, 1))?;
                let sin = freq_cis_imag.reshape((cfg.seq_len, cfg.head_size() / 2, 1))?;
                Ok(Self {
                    masks: Arc::new(Mutex::new(HashMap::new())),
                    use_kv_cache,
                    kvs: Arc::new(Mutex::new(::alloc::vec::from_elem(None, cfg.n_layers))),
                    cos,
                    sin,
                    device: vb.device().clone(),
                })
            }
            fn mask(&self, t: usize) -> Result<Tensor> {
                let mut masks = self.masks.lock().unwrap();
                if let Some(mask) = masks.get(&t) {
                    Ok(mask.clone())
                } else {
                    let mask: Vec<_> = (0..t)
                        .flat_map(|i| (0..t).map(move |j| u8::from(j > i)))
                        .collect();
                    let mask = Tensor::from_slice(&mask, (t, t), &self.device)?;
                    masks.insert(t, mask.clone());
                    Ok(mask)
                }
            }
        }
        fn silu(xs: &Tensor) -> Result<Tensor> {
            xs / (xs.neg()?.exp()? + 1.0)?
        }
        struct CausalSelfAttention {
            q_proj: Linear,
            k_proj: Linear,
            v_proj: Linear,
            o_proj: Linear,
            n_head: usize,
            n_key_value_head: usize,
            head_dim: usize,
            cache: Cache,
        }
        impl CausalSelfAttention {
            fn apply_rotary_emb(&self, x: &Tensor, index_pos: usize) -> Result<Tensor> {
                let (b_sz, seq_len, h, n_embd) = x.dims4()?;
                let cos = self.cache.cos.i(index_pos..index_pos + seq_len)?;
                let sin = self.cache.sin.i(index_pos..index_pos + seq_len)?;
                let cos = cos.unsqueeze(1)?;
                let sin = sin.unsqueeze(1)?;
                let cos = cos.broadcast_as((b_sz, seq_len, 1, n_embd / 2, 1))?;
                let sin = sin.broadcast_as((b_sz, seq_len, 1, n_embd / 2, 1))?;
                let x = x.reshape((b_sz, seq_len, h, n_embd / 2, 2))?;
                let x0 = x.narrow(D::Minus1, 0, 1)?;
                let x1 = x.narrow(D::Minus1, 1, 1)?;
                let dst0 = (x0.broadcast_mul(&cos)? - x1.broadcast_mul(&sin)?)?;
                let dst1 = (x0.broadcast_mul(&sin)? + x1.broadcast_mul(&cos)?)?;
                let rope =
                    Tensor::cat(&[&dst0, &dst1], D::Minus1)?.reshape((b_sz, seq_len, h, n_embd))?;
                Ok(rope)
            }
            fn forward(&self, x: &Tensor, index_pos: usize, block_idx: usize) -> Result<Tensor> {
                let (b_sz, seq_len, n_embd) = x.dims3()?;
                let q = self.q_proj.forward(x)?;
                let k = self.k_proj.forward(x)?;
                let v = self.v_proj.forward(x)?;
                let q = q.reshape((b_sz, seq_len, self.n_head, self.head_dim))?;
                let k = k.reshape((b_sz, seq_len, self.n_key_value_head, self.head_dim))?;
                let mut v = v.reshape((b_sz, seq_len, self.n_key_value_head, self.head_dim))?;
                let q = self.apply_rotary_emb(&q, index_pos)?;
                let mut k = self.apply_rotary_emb(&k, index_pos)?;
                if self.cache.use_kv_cache {
                    let mut cache = self.cache.kvs.lock().unwrap();
                    if let Some((cache_k, cache_v)) = &cache[block_idx] {
                        k = Tensor::cat(&[cache_k, &k], 1)?.contiguous()?;
                        v = Tensor::cat(&[cache_v, &v], 1)?.contiguous()?;
                    }
                    cache[block_idx] = Some((k.clone(), v.clone()));
                }
                let k = self.repeat_kv(k)?;
                let v = self.repeat_kv(v)?;
                let q = q.transpose(1, 2)?.contiguous()?;
                let k = k.transpose(1, 2)?.contiguous()?;
                let v = v.transpose(1, 2)?.contiguous()?;
                let att = (q.matmul(&k.t()?)? / (self.head_dim as f64).sqrt())?;
                let mask = self.cache.mask(seq_len)?.broadcast_as(att.shape())?;
                let att = masked_fill(&att, &mask, f32::NEG_INFINITY)?;
                let att = candle_nn::ops::softmax(&att, D::Minus1)?;
                let y = att.matmul(&v.contiguous()?)?;
                let y = y.transpose(1, 2)?.reshape(&[b_sz, seq_len, n_embd])?;
                let y = self.o_proj.forward(&y)?;
                Ok(y)
            }
            fn repeat_kv(&self, x: Tensor) -> Result<Tensor> {
                let n_rep = self.n_head / self.n_key_value_head;
                if n_rep == 1 {
                    Ok(x)
                } else {
                    let (b_sz, seq_len, n_kv_head, head_dim) = x.dims4()?;
                    let x = x
                        .unsqueeze(3)?
                        .expand((b_sz, seq_len, n_kv_head, n_rep, head_dim))?
                        .reshape((b_sz, seq_len, n_kv_head * n_rep, head_dim))?;
                    Ok(x)
                }
            }
            fn load(vb: VarBuilder, cache: &Cache, cfg: &Config) -> Result<Self> {
                let size_in = cfg.dim;
                let size_q = (cfg.dim / cfg.n_heads) * cfg.n_heads;
                let size_kv = (cfg.dim / cfg.n_heads) * cfg.n_kv_heads;
                let q_proj = linear(size_in, size_q, vb.pp("q_proj"))?;
                let k_proj = linear(size_in, size_kv, vb.pp("k_proj"))?;
                let v_proj = linear(size_in, size_kv, vb.pp("v_proj"))?;
                let o_proj = linear(size_q, size_in, vb.pp("o_proj"))?;
                Ok(Self {
                    q_proj,
                    k_proj,
                    v_proj,
                    o_proj,
                    n_head: cfg.n_heads,
                    n_key_value_head: cfg.n_kv_heads,
                    head_dim: cfg.dim / cfg.n_heads,
                    cache: cache.clone(),
                })
            }
        }
        fn masked_fill(on_false: &Tensor, mask: &Tensor, on_true: f32) -> Result<Tensor> {
            let shape = mask.shape();
            let on_true = Tensor::new(on_true, on_false.device())?.broadcast_as(shape.dims())?;
            let m = mask.where_cond(&on_true, on_false)?;
            Ok(m)
        }
        struct Mlp {
            c_fc1: Linear,
            c_fc2: Linear,
            c_proj: Linear,
        }
        impl Mlp {
            fn new(c_fc1: Linear, c_fc2: Linear, c_proj: Linear) -> Self {
                Self {
                    c_fc1,
                    c_fc2,
                    c_proj,
                }
            }
            fn forward(&self, x: &Tensor) -> Result<Tensor> {
                let x = (silu(&self.c_fc1.forward(x)?)? * self.c_fc2.forward(x)?)?;
                self.c_proj.forward(&x)
            }
            fn load(vb: VarBuilder, cfg: &Config) -> Result<Self> {
                let h_size = cfg.dim;
                let i_size = cfg.hidden_dim;
                let c_fc1 = linear(h_size, i_size, vb.pp("gate_proj"))?;
                let c_fc2 = linear(h_size, i_size, vb.pp("up_proj"))?;
                let c_proj = linear(i_size, h_size, vb.pp("down_proj"))?;
                Ok(Self::new(c_fc1, c_fc2, c_proj))
            }
        }
        struct Block {
            rms_1: RmsNorm,
            attn: CausalSelfAttention,
            rms_2: RmsNorm,
            mlp: Mlp,
        }
        impl Block {
            fn new(rms_1: RmsNorm, attn: CausalSelfAttention, rms_2: RmsNorm, mlp: Mlp) -> Self {
                Self {
                    rms_1,
                    attn,
                    rms_2,
                    mlp,
                }
            }
            fn forward(&self, x: &Tensor, index_pos: usize, block_idx: usize) -> Result<Tensor> {
                let residual = x;
                let x = self.rms_1.forward(x)?;
                let x = (self.attn.forward(&x, index_pos, block_idx)? + residual)?;
                let residual = &x;
                let x = (self.mlp.forward(&self.rms_2.forward(&x)?)? + residual)?;
                Ok(x)
            }
            fn load(vb: VarBuilder, cache: &Cache, cfg: &Config) -> Result<Self> {
                let attn = CausalSelfAttention::load(vb.pp("self_attn"), cache, cfg)?;
                let mlp = Mlp::load(vb.pp("mlp"), cfg)?;
                let input_layernorm = rms_norm(cfg.dim, cfg.norm_eps, vb.pp("input_layernorm"))?;
                let post_attention_layernorm =
                    rms_norm(cfg.dim, cfg.norm_eps, vb.pp("post_attention_layernorm"))?;
                Ok(Self::new(
                    input_layernorm,
                    attn,
                    post_attention_layernorm,
                    mlp,
                ))
            }
        }
        pub(crate) struct Llama {
            inner: Arc<LlamaInner>,
        }
        #[automatically_derived]
        impl ::core::clone::Clone for Llama {
            #[inline]
            fn clone(&self) -> Llama {
                Llama {
                    inner: ::core::clone::Clone::clone(&self.inner),
                }
            }
        }
        struct LlamaInner {
            wte: Embedding,
            blocks: Vec<Block>,
            ln_f: RmsNorm,
            lm_head: Linear,
            pub(crate) config: Config,
        }
        impl Llama {
            pub(crate) fn forward(&self, x: &Tensor, index_pos: usize) -> Result<Tensor> {
                let (_b_sz, _seq_len) = x.dims2()?;
                let mut x = self.inner.wte.forward(x)?;
                for (block_idx, block) in self.inner.blocks.iter().enumerate() {
                    x = block.forward(&x, index_pos, block_idx)?;
                }
                let x = self.inner.ln_f.forward(&x)?;
                let logits = self.inner.lm_head.forward(&x)?;
                logits.to_dtype(DType::F32)
            }
            pub(crate) fn seq_len(&self) -> usize {
                self.inner.config.seq_len
            }
            pub(crate) fn load(vb: VarBuilder, cache: &Cache, cfg: Config) -> Result<Self> {
                let wte = embedding(cfg.vocab_size, cfg.dim, vb.pp("model.embed_tokens"))?;
                let lm_head = linear(cfg.dim, cfg.vocab_size, vb.pp("lm_head"))?;
                let ln_f = rms_norm(cfg.dim, cfg.norm_eps, vb.pp("model.norm"))?;
                let blocks: Vec<_> = (0..cfg.n_layers)
                    .map(|i| {
                        Block::load(
                            vb.pp(&{
                                let res = ::alloc::fmt::format(format_args!("model.layers.{0}", i));
                                res
                            }),
                            cache,
                            &cfg,
                        )
                        .unwrap()
                    })
                    .collect();
                Ok(Self {
                    inner: Arc::new(LlamaInner {
                        wte,
                        blocks,
                        ln_f,
                        lm_head,
                        config: cfg,
                    }),
                })
            }
        }
    }
    mod weights {
        use super::model::Config;
        use anyhow::Result;
        use byteorder::{LittleEndian, ReadBytesExt};
        use candle_core::{DType, Device, IndexOp, Shape, Tensor};
        use candle_nn::VarBuilder;
        pub(crate) struct TransformerWeights {
            token_embedding_table: Tensor,
            rms_att_weight: Tensor,
            rms_ffn_weight: Tensor,
            wq: Tensor,
            wk: Tensor,
            wv: Tensor,
            wo: Tensor,
            w1: Tensor,
            w2: Tensor,
            w3: Tensor,
            rms_final_weight: Tensor,
            freq_cis_real: Tensor,
            freq_cis_imag: Tensor,
        }
        fn read_i32<R: std::io::Read>(r: &mut R) -> Result<i32> {
            let mut buf = [0u8; 4];
            r.read_exact(&mut buf)?;
            Ok(i32::from_le_bytes(buf))
        }
        fn read_tensor<R: std::io::Read, S: Into<Shape>>(
            r: &mut R,
            shape: S,
            dev: &Device,
        ) -> Result<Tensor> {
            let shape = shape.into();
            let mut data_t = ::alloc::vec::from_elem(0f32, shape.elem_count());
            r.read_f32_into::<LittleEndian>(&mut data_t)?;
            let tensor = Tensor::from_vec(data_t, shape, dev)?;
            Ok(tensor)
        }
        impl Config {
            pub(crate) fn from_reader<R: std::io::Read>(r: &mut R) -> Result<Self> {
                let dim = read_i32(r)? as usize;
                let hidden_dim = read_i32(r)? as usize;
                let n_layers = read_i32(r)? as usize;
                let n_heads = read_i32(r)? as usize;
                let n_kv_heads = read_i32(r)? as usize;
                let vocab_size = read_i32(r)? as usize;
                let seq_len = read_i32(r)? as usize;
                Ok(Self {
                    dim,
                    hidden_dim,
                    n_layers,
                    n_heads,
                    n_kv_heads,
                    vocab_size,
                    seq_len,
                    norm_eps: 1e-5,
                })
            }
            pub(crate) fn head_size(&self) -> usize {
                self.dim / self.n_heads
            }
        }
        impl TransformerWeights {
            pub(crate) fn from_reader<R: std::io::Read>(
                r: &mut R,
                c: &Config,
                dev: &Device,
            ) -> Result<Self> {
                let token_embedding_table = read_tensor(r, (c.vocab_size, c.dim), dev)?;
                let rms_att_weight = read_tensor(r, (c.n_layers, c.dim), dev)?;
                let wq = read_tensor(r, (c.n_layers, c.dim, c.dim), dev)?;
                let wk = read_tensor(r, (c.n_layers, c.dim, c.dim), dev)?;
                let wv = read_tensor(r, (c.n_layers, c.dim, c.dim), dev)?;
                let wo = read_tensor(r, (c.n_layers, c.dim, c.dim), dev)?;
                let rms_ffn_weight = read_tensor(r, (c.n_layers, c.dim), dev)?;
                let w1 = read_tensor(r, (c.n_layers, c.hidden_dim, c.dim), dev)?;
                let w2 = read_tensor(r, (c.n_layers, c.dim, c.hidden_dim), dev)?;
                let w3 = read_tensor(r, (c.n_layers, c.hidden_dim, c.dim), dev)?;
                let rms_final_weight = read_tensor(r, c.dim, dev)?;
                let head_size = c.head_size();
                let freq_cis_real = read_tensor(r, (c.seq_len, head_size / 2), dev)?;
                let freq_cis_imag = read_tensor(r, (c.seq_len, head_size / 2), dev)?;
                Ok(Self {
                    token_embedding_table,
                    rms_att_weight,
                    wq,
                    wk,
                    wv,
                    wo,
                    rms_ffn_weight,
                    w1,
                    w2,
                    w3,
                    rms_final_weight,
                    freq_cis_real,
                    freq_cis_imag,
                })
            }
            pub(crate) fn var_builder(
                &self,
                cfg: &Config,
                device: &Device,
            ) -> Result<VarBuilder<'static>> {
                let tr = device.is_cpu() && !candle_core::utils::has_mkl();
                let tr = |x: Tensor| if tr { x.t()?.contiguous()?.t() } else { Ok(x) };
                let mut ws = std::collections::HashMap::new();
                let mut insert = |name: &str, t: Tensor| {
                    ws.insert(name.to_string(), t);
                };
                insert("rot.freq_cis_real", self.freq_cis_real.clone());
                insert("rot.freq_cis_imag", self.freq_cis_imag.clone());
                insert(
                    "model.embed_tokens.weight",
                    self.token_embedding_table.clone(),
                );
                insert("lm_head.weight", tr(self.token_embedding_table.clone())?);
                insert("model.norm.weight", self.rms_final_weight.clone());
                for layer in 0..cfg.n_layers {
                    ws.insert(
                        {
                            let res = ::alloc::fmt::format(format_args!(
                                "model.layers.{0}.self_attn.q_proj.weight",
                                layer,
                            ));
                            res
                        },
                        tr(self.wq.i(layer)?)?,
                    );
                    ws.insert(
                        {
                            let res = ::alloc::fmt::format(format_args!(
                                "model.layers.{0}.self_attn.k_proj.weight",
                                layer,
                            ));
                            res
                        },
                        tr(self.wk.i(layer)?)?,
                    );
                    ws.insert(
                        {
                            let res = ::alloc::fmt::format(format_args!(
                                "model.layers.{0}.self_attn.v_proj.weight",
                                layer,
                            ));
                            res
                        },
                        tr(self.wv.i(layer)?)?,
                    );
                    ws.insert(
                        {
                            let res = ::alloc::fmt::format(format_args!(
                                "model.layers.{0}.self_attn.o_proj.weight",
                                layer,
                            ));
                            res
                        },
                        tr(self.wo.i(layer)?)?,
                    );
                    ws.insert(
                        {
                            let res = ::alloc::fmt::format(format_args!(
                                "model.layers.{0}.mlp.gate_proj.weight",
                                layer
                            ));
                            res
                        },
                        tr(self.w1.i(layer)?)?,
                    );
                    ws.insert(
                        {
                            let res = ::alloc::fmt::format(format_args!(
                                "model.layers.{0}.mlp.down_proj.weight",
                                layer
                            ));
                            res
                        },
                        tr(self.w2.i(layer)?)?,
                    );
                    ws.insert(
                        {
                            let res = ::alloc::fmt::format(format_args!(
                                "model.layers.{0}.mlp.up_proj.weight",
                                layer
                            ));
                            res
                        },
                        tr(self.w3.i(layer)?)?,
                    );
                    ws.insert(
                        {
                            let res = ::alloc::fmt::format(format_args!(
                                "model.layers.{0}.input_layernorm.weight",
                                layer,
                            ));
                            res
                        },
                        self.rms_att_weight.i(layer)?,
                    );
                    ws.insert(
                        {
                            let res = ::alloc::fmt::format(format_args!(
                                "model.layers.{0}.post_attention_layernorm.weight",
                                layer,
                            ));
                            res
                        },
                        self.rms_ffn_weight.i(layer)?,
                    );
                }
                let vb = VarBuilder::from_tensors(ws, DType::F32, device);
                Ok(vb)
            }
        }
    }
    use anyhow::Result;
    use candle_core::{IndexOp, Tensor};
    use candle_transformers::generation::LogitsProcessor;
    use model::Cache;
    use model::{Config, Llama};
    use std::io::Write;
    use tokenizers::{Token, Tokenizer};
    use weights::TransformerWeights;
    const MODEL: ::std::thread::LocalKey<std::cell::UnsafeCell<Option<Llama>>> = {
        #[inline]
        fn __init() -> std::cell::UnsafeCell<Option<Llama>> {
            std::cell::UnsafeCell::new(None)
        }
        #[inline]
        unsafe fn __getit(
            init: ::std::option::Option<
                &mut ::std::option::Option<std::cell::UnsafeCell<Option<Llama>>>,
            >,
        ) -> ::std::option::Option<&'static std::cell::UnsafeCell<Option<Llama>>> {
            #[thread_local]
            static __KEY: ::std::thread::local_impl::Key<std::cell::UnsafeCell<Option<Llama>>> =
                ::std::thread::local_impl::Key::<std::cell::UnsafeCell<Option<Llama>>>::new();
            #[allow(unused_unsafe)]
            unsafe {
                __KEY.get(move || {
                    if let ::std::option::Option::Some(init) = init {
                        if let ::std::option::Option::Some(value) = init.take() {
                            return value;
                        } else if true {
                            {
                                ::core::panicking::panic_fmt(format_args!(
                                    "internal error: entered unreachable code: {0}",
                                    format_args!("missing default value"),
                                ));
                            };
                        }
                    }
                    __init()
                })
            }
        }
        unsafe { ::std::thread::LocalKey::new(__getit) }
    };
    const TOKENIZER: ::std::thread::LocalKey<std::cell::UnsafeCell<Option<Tokenizer>>> = {
        #[inline]
        fn __init() -> std::cell::UnsafeCell<Option<Tokenizer>> {
            std::cell::UnsafeCell::new(None)
        }
        #[inline]
        unsafe fn __getit(
            init: ::std::option::Option<
                &mut ::std::option::Option<std::cell::UnsafeCell<Option<Tokenizer>>>,
            >,
        ) -> ::std::option::Option<&'static std::cell::UnsafeCell<Option<Tokenizer>>> {
            #[thread_local]
            static __KEY: ::std::thread::local_impl::Key<std::cell::UnsafeCell<Option<Tokenizer>>> =
                ::std::thread::local_impl::Key::<std::cell::UnsafeCell<Option<Tokenizer>>>::new();
            #[allow(unused_unsafe)]
            unsafe {
                __KEY.get(move || {
                    if let ::std::option::Option::Some(init) = init {
                        if let ::std::option::Option::Some(value) = init.take() {
                            return value;
                        } else if true {
                            {
                                ::core::panicking::panic_fmt(format_args!(
                                    "internal error: entered unreachable code: {0}",
                                    format_args!("missing default value"),
                                ));
                            };
                        }
                    }
                    __init()
                })
            }
        }
        unsafe { ::std::thread::LocalKey::new(__getit) }
    };
    pub(crate) struct Args {
        prompt: String,
        /// The temperature used to generate samples.
        temperature: Option<f64>,
        /// Nucleus sampling probability cutoff.
        top_p: Option<f64>,
        /// Penalty to be applied for repeating tokens, 1. means no penalty.
        repeat_penalty: f32,
        /// The context size to consider for the repeat penalty.
        repeat_last_n: usize,
    }
    pub(super) fn load_model(file: &str) -> Result<Llama> {
        MODEL.with(|model| {
            let model = unsafe { &mut *model.get() };
            if model.is_none() {
                *model = Some(load_model_inner(file)?);
            } else {
                {
                    ::std::io::_print(format_args!("model already loaded\n"));
                };
            }
            Ok(model.as_ref().unwrap().clone())
        })
    }
    fn load_model_inner(file: &str) -> Result<Llama> {
        let mut file = std::fs::File::open(file)?;
        let config = Config::from_reader(&mut file)?;
        let device = candle_core::Device::Cpu;
        let weights = TransformerWeights::from_reader(&mut file, &config, &device)?;
        let vb = weights.var_builder(&config, &device)?;
        let cache = Cache::new(true, &config, vb.pp("rot"))?;
        let model = Llama::load(vb, &cache, config)?;
        Ok(model)
    }
    pub(super) fn load_tokenizer(file: &str) -> Result<Tokenizer> {
        TOKENIZER.with(|tokenizer| {
            let tokenizer = unsafe { &mut *tokenizer.get() };
            if tokenizer.is_none() {
                *tokenizer = Some(load_tokenizer_inner(file)?);
            } else {
                {
                    ::std::io::_print(format_args!("tokenizer already loaded\n"));
                };
            }
            Ok(tokenizer.as_ref().unwrap().clone())
        })
    }
    fn load_tokenizer_inner(file: &str) -> Result<Tokenizer> {
        let bytes = std::fs::read(file)?;
        Tokenizer::from_bytes(&bytes).map_err(|e| {
            ::anyhow::__private::must_use({
                use ::anyhow::__private::kind::*;
                let error = match e {
                    error => (&error).anyhow_kind().new(error),
                };
                error
            })
        })
    }
    pub(crate) fn run_inference(model: Llama, tokenizer: Tokenizer, args: Args) -> Result<()> {
        {
            ::std::io::_print(format_args!("starting the inference loop\n"));
        };
        let mut logits_processor = LogitsProcessor::new(299792458, args.temperature, args.top_p);
        let mut index_pos = 0;
        {
            ::std::io::_print(format_args!("{0}", args.prompt));
        };
        let mut tokens = tokenizer
            .encode(args.prompt.clone(), true)
            .map_err(anyhow::Error::msg)?
            .get_ids()
            .to_vec();
        let device = candle_core::Device::Cpu;
        let start_gen = std::time::Instant::now();
        for index in 0.. {
            if tokens.len() >= model.seq_len() {
                break;
            }
            let context_size = if index > 0 { 1 } else { tokens.len() };
            let ctxt = &tokens[tokens.len().saturating_sub(context_size)..];
            let input = Tensor::new(ctxt, &device)?.unsqueeze(0)?;
            let logits = model.forward(&input, index_pos)?;
            let logits = logits.i((0, logits.dim(1)? - 1))?;
            let logits = if args.repeat_penalty == 1. || tokens.is_empty() {
                logits
            } else {
                let start_at = tokens.len().saturating_sub(args.repeat_last_n);
                candle_transformers::utils::apply_repeat_penalty(
                    &logits,
                    args.repeat_penalty,
                    &tokens[start_at..],
                )?
            };
            index_pos += ctxt.len();
            let next_token = logits_processor.sample(&logits)?;
            tokens.push(next_token);
            if let Some(text) = tokenizer.id_to_token(next_token) {
                let text = text.replace('▁', " ").replace("<0x0A>", "\n");
                {
                    ::std::io::_print(format_args!("{0}", text));
                };
                std::io::stdout().flush()?;
            }
        }
        let dt = start_gen.elapsed();
        {
            ::std::io::_print(format_args!(
                "\n{0} tokens generated ({1:.2} token/s)\n\n",
                tokens.len(),
                tokens.len() as f64 / dt.as_secs_f64(),
            ));
        };
        Ok(())
    }
}
mod wick {
    pub use async_trait::async_trait;
    pub use wick_component::flow_component::Context;
    #[allow(unused)]
    pub(crate) use wick_component::WickStream;
    #[allow(unused)]
    pub(crate) use wick_component::*;
    #[allow(clippy::exhaustive_structs)]
    pub struct RootConfig {
        #[serde(rename = "model_dir")]
        pub model_dir: String,
        #[serde(rename = "model")]
        pub model: String,
        #[serde(rename = "tokenizer")]
        pub tokenizer: String,
    }
    #[automatically_derived]
    #[allow(clippy::exhaustive_structs)]
    impl ::core::fmt::Debug for RootConfig {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_struct_field3_finish(
                f,
                "RootConfig",
                "model_dir",
                &self.model_dir,
                "model",
                &self.model,
                "tokenizer",
                &&self.tokenizer,
            )
        }
    }
    #[automatically_derived]
    #[allow(clippy::exhaustive_structs)]
    impl ::core::clone::Clone for RootConfig {
        #[inline]
        fn clone(&self) -> RootConfig {
            RootConfig {
                model_dir: ::core::clone::Clone::clone(&self.model_dir),
                model: ::core::clone::Clone::clone(&self.model),
                tokenizer: ::core::clone::Clone::clone(&self.tokenizer),
            }
        }
    }
    #[doc(hidden)]
    #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
    const _: () = {
        #[allow(unused_extern_crates, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl _serde::Serialize for RootConfig {
            fn serialize<__S>(
                &self,
                __serializer: __S,
            ) -> _serde::__private::Result<__S::Ok, __S::Error>
            where
                __S: _serde::Serializer,
            {
                let mut __serde_state = _serde::Serializer::serialize_struct(
                    __serializer,
                    "RootConfig",
                    false as usize + 1 + 1 + 1,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "model_dir",
                    &self.model_dir,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "model",
                    &self.model,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "tokenizer",
                    &self.tokenizer,
                )?;
                _serde::ser::SerializeStruct::end(__serde_state)
            }
        }
    };
    #[doc(hidden)]
    #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
    const _: () = {
        #[allow(unused_extern_crates, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl<'de> _serde::Deserialize<'de> for RootConfig {
            fn deserialize<__D>(__deserializer: __D) -> _serde::__private::Result<Self, __D::Error>
            where
                __D: _serde::Deserializer<'de>,
            {
                #[allow(non_camel_case_types)]
                #[doc(hidden)]
                enum __Field {
                    __field0,
                    __field1,
                    __field2,
                    __ignore,
                }
                #[doc(hidden)]
                struct __FieldVisitor;
                impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                    type Value = __Field;
                    fn expecting(
                        &self,
                        __formatter: &mut _serde::__private::Formatter,
                    ) -> _serde::__private::fmt::Result {
                        _serde::__private::Formatter::write_str(__formatter, "field identifier")
                    }
                    fn visit_u64<__E>(
                        self,
                        __value: u64,
                    ) -> _serde::__private::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            0u64 => _serde::__private::Ok(__Field::__field0),
                            1u64 => _serde::__private::Ok(__Field::__field1),
                            2u64 => _serde::__private::Ok(__Field::__field2),
                            _ => _serde::__private::Ok(__Field::__ignore),
                        }
                    }
                    fn visit_str<__E>(
                        self,
                        __value: &str,
                    ) -> _serde::__private::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            "model_dir" => _serde::__private::Ok(__Field::__field0),
                            "model" => _serde::__private::Ok(__Field::__field1),
                            "tokenizer" => _serde::__private::Ok(__Field::__field2),
                            _ => _serde::__private::Ok(__Field::__ignore),
                        }
                    }
                    fn visit_bytes<__E>(
                        self,
                        __value: &[u8],
                    ) -> _serde::__private::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            b"model_dir" => _serde::__private::Ok(__Field::__field0),
                            b"model" => _serde::__private::Ok(__Field::__field1),
                            b"tokenizer" => _serde::__private::Ok(__Field::__field2),
                            _ => _serde::__private::Ok(__Field::__ignore),
                        }
                    }
                }
                impl<'de> _serde::Deserialize<'de> for __Field {
                    #[inline]
                    fn deserialize<__D>(
                        __deserializer: __D,
                    ) -> _serde::__private::Result<Self, __D::Error>
                    where
                        __D: _serde::Deserializer<'de>,
                    {
                        _serde::Deserializer::deserialize_identifier(__deserializer, __FieldVisitor)
                    }
                }
                #[doc(hidden)]
                struct __Visitor<'de> {
                    marker: _serde::__private::PhantomData<RootConfig>,
                    lifetime: _serde::__private::PhantomData<&'de ()>,
                }
                impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                    type Value = RootConfig;
                    fn expecting(
                        &self,
                        __formatter: &mut _serde::__private::Formatter,
                    ) -> _serde::__private::fmt::Result {
                        _serde::__private::Formatter::write_str(__formatter, "struct RootConfig")
                    }
                    #[inline]
                    fn visit_seq<__A>(
                        self,
                        mut __seq: __A,
                    ) -> _serde::__private::Result<Self::Value, __A::Error>
                    where
                        __A: _serde::de::SeqAccess<'de>,
                    {
                        let __field0 =
                            match _serde::de::SeqAccess::next_element::<String>(&mut __seq)? {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            0usize,
                                            &"struct RootConfig with 3 elements",
                                        ),
                                    );
                                }
                            };
                        let __field1 =
                            match _serde::de::SeqAccess::next_element::<String>(&mut __seq)? {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            1usize,
                                            &"struct RootConfig with 3 elements",
                                        ),
                                    );
                                }
                            };
                        let __field2 =
                            match _serde::de::SeqAccess::next_element::<String>(&mut __seq)? {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            2usize,
                                            &"struct RootConfig with 3 elements",
                                        ),
                                    );
                                }
                            };
                        _serde::__private::Ok(RootConfig {
                            model_dir: __field0,
                            model: __field1,
                            tokenizer: __field2,
                        })
                    }
                    #[inline]
                    fn visit_map<__A>(
                        self,
                        mut __map: __A,
                    ) -> _serde::__private::Result<Self::Value, __A::Error>
                    where
                        __A: _serde::de::MapAccess<'de>,
                    {
                        let mut __field0: _serde::__private::Option<String> =
                            _serde::__private::None;
                        let mut __field1: _serde::__private::Option<String> =
                            _serde::__private::None;
                        let mut __field2: _serde::__private::Option<String> =
                            _serde::__private::None;
                        while let _serde::__private::Some(__key) =
                            _serde::de::MapAccess::next_key::<__Field>(&mut __map)?
                        {
                            match __key {
                                __Field::__field0 => {
                                    if _serde::__private::Option::is_some(&__field0) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "model_dir",
                                            ),
                                        );
                                    }
                                    __field0 = _serde::__private::Some(
                                        _serde::de::MapAccess::next_value::<String>(&mut __map)?,
                                    );
                                }
                                __Field::__field1 => {
                                    if _serde::__private::Option::is_some(&__field1) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "model",
                                            ),
                                        );
                                    }
                                    __field1 = _serde::__private::Some(
                                        _serde::de::MapAccess::next_value::<String>(&mut __map)?,
                                    );
                                }
                                __Field::__field2 => {
                                    if _serde::__private::Option::is_some(&__field2) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "tokenizer",
                                            ),
                                        );
                                    }
                                    __field2 = _serde::__private::Some(
                                        _serde::de::MapAccess::next_value::<String>(&mut __map)?,
                                    );
                                }
                                _ => {
                                    let _ = _serde::de::MapAccess::next_value::<
                                        _serde::de::IgnoredAny,
                                    >(&mut __map)?;
                                }
                            }
                        }
                        let __field0 = match __field0 {
                            _serde::__private::Some(__field0) => __field0,
                            _serde::__private::None => {
                                _serde::__private::de::missing_field("model_dir")?
                            }
                        };
                        let __field1 = match __field1 {
                            _serde::__private::Some(__field1) => __field1,
                            _serde::__private::None => {
                                _serde::__private::de::missing_field("model")?
                            }
                        };
                        let __field2 = match __field2 {
                            _serde::__private::Some(__field2) => __field2,
                            _serde::__private::None => {
                                _serde::__private::de::missing_field("tokenizer")?
                            }
                        };
                        _serde::__private::Ok(RootConfig {
                            model_dir: __field0,
                            model: __field1,
                            tokenizer: __field2,
                        })
                    }
                }
                #[doc(hidden)]
                const FIELDS: &'static [&'static str] = &["model_dir", "model", "tokenizer"];
                _serde::Deserializer::deserialize_struct(
                    __deserializer,
                    "RootConfig",
                    FIELDS,
                    __Visitor {
                        marker: _serde::__private::PhantomData::<RootConfig>,
                        lifetime: _serde::__private::PhantomData,
                    },
                )
            }
        }
    };
    #[allow(clippy::exhaustive_structs)]
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for RootConfig {}
    #[automatically_derived]
    #[allow(clippy::exhaustive_structs)]
    impl ::core::cmp::PartialEq for RootConfig {
        #[inline]
        fn eq(&self, other: &RootConfig) -> bool {
            self.model_dir == other.model_dir
                && self.model == other.model
                && self.tokenizer == other.tokenizer
        }
    }
    impl Default for RootConfig {
        fn default() -> Self {
            Self {
                model_dir: Default::default(),
                model: Default::default(),
                tokenizer: Default::default(),
            }
        }
    }
    pub(crate) trait RootConfigContext {
        fn root_config(&self) -> &'static RootConfig;
    }
    impl<T> RootConfigContext for Context<T>
    where
        T: std::fmt::Debug + wick_component::flow_component::LocalAwareSend,
    {
        fn root_config(&self) -> &'static RootConfig {
            #[cfg(not(target_family = "wasm"))]
            {
                {
                    ::core::panicking::panic_fmt(format_args!(
                        "not implemented: {0}",
                        format_args!("root_config is only available in wasm builds"),
                    ));
                }
            }
        }
    }
    ///Additional generated types
    pub mod types {
        #[allow(unused)]
        use super::types;
    }
    ///Types associated with the `generate` operation
    pub mod generate {
        #[allow(unused)]
        use super::*;
        #[allow(clippy::exhaustive_structs)]
        pub struct Config {}
        #[automatically_derived]
        #[allow(clippy::exhaustive_structs)]
        impl ::core::fmt::Debug for Config {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::write_str(f, "Config")
            }
        }
        #[automatically_derived]
        #[allow(clippy::exhaustive_structs)]
        impl ::core::clone::Clone for Config {
            #[inline]
            fn clone(&self) -> Config {
                Config {}
            }
        }
        #[automatically_derived]
        #[allow(clippy::exhaustive_structs)]
        impl ::core::default::Default for Config {
            #[inline]
            fn default() -> Config {
                Config {}
            }
        }
        #[doc(hidden)]
        #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
        const _: () = {
            #[allow(unused_extern_crates, clippy::useless_attribute)]
            extern crate serde as _serde;
            #[automatically_derived]
            impl _serde::Serialize for Config {
                fn serialize<__S>(
                    &self,
                    __serializer: __S,
                ) -> _serde::__private::Result<__S::Ok, __S::Error>
                where
                    __S: _serde::Serializer,
                {
                    let __serde_state = _serde::Serializer::serialize_struct(
                        __serializer,
                        "Config",
                        false as usize,
                    )?;
                    _serde::ser::SerializeStruct::end(__serde_state)
                }
            }
        };
        #[doc(hidden)]
        #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
        const _: () = {
            #[allow(unused_extern_crates, clippy::useless_attribute)]
            extern crate serde as _serde;
            #[automatically_derived]
            impl<'de> _serde::Deserialize<'de> for Config {
                fn deserialize<__D>(
                    __deserializer: __D,
                ) -> _serde::__private::Result<Self, __D::Error>
                where
                    __D: _serde::Deserializer<'de>,
                {
                    #[allow(non_camel_case_types)]
                    #[doc(hidden)]
                    enum __Field {
                        __ignore,
                    }
                    #[doc(hidden)]
                    struct __FieldVisitor;
                    impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                        type Value = __Field;
                        fn expecting(
                            &self,
                            __formatter: &mut _serde::__private::Formatter,
                        ) -> _serde::__private::fmt::Result {
                            _serde::__private::Formatter::write_str(__formatter, "field identifier")
                        }
                        fn visit_u64<__E>(
                            self,
                            __value: u64,
                        ) -> _serde::__private::Result<Self::Value, __E>
                        where
                            __E: _serde::de::Error,
                        {
                            match __value {
                                _ => _serde::__private::Ok(__Field::__ignore),
                            }
                        }
                        fn visit_str<__E>(
                            self,
                            __value: &str,
                        ) -> _serde::__private::Result<Self::Value, __E>
                        where
                            __E: _serde::de::Error,
                        {
                            match __value {
                                _ => _serde::__private::Ok(__Field::__ignore),
                            }
                        }
                        fn visit_bytes<__E>(
                            self,
                            __value: &[u8],
                        ) -> _serde::__private::Result<Self::Value, __E>
                        where
                            __E: _serde::de::Error,
                        {
                            match __value {
                                _ => _serde::__private::Ok(__Field::__ignore),
                            }
                        }
                    }
                    impl<'de> _serde::Deserialize<'de> for __Field {
                        #[inline]
                        fn deserialize<__D>(
                            __deserializer: __D,
                        ) -> _serde::__private::Result<Self, __D::Error>
                        where
                            __D: _serde::Deserializer<'de>,
                        {
                            _serde::Deserializer::deserialize_identifier(
                                __deserializer,
                                __FieldVisitor,
                            )
                        }
                    }
                    #[doc(hidden)]
                    struct __Visitor<'de> {
                        marker: _serde::__private::PhantomData<Config>,
                        lifetime: _serde::__private::PhantomData<&'de ()>,
                    }
                    impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                        type Value = Config;
                        fn expecting(
                            &self,
                            __formatter: &mut _serde::__private::Formatter,
                        ) -> _serde::__private::fmt::Result {
                            _serde::__private::Formatter::write_str(__formatter, "struct Config")
                        }
                        #[inline]
                        fn visit_seq<__A>(
                            self,
                            _: __A,
                        ) -> _serde::__private::Result<Self::Value, __A::Error>
                        where
                            __A: _serde::de::SeqAccess<'de>,
                        {
                            _serde::__private::Ok(Config {})
                        }
                        #[inline]
                        fn visit_map<__A>(
                            self,
                            mut __map: __A,
                        ) -> _serde::__private::Result<Self::Value, __A::Error>
                        where
                            __A: _serde::de::MapAccess<'de>,
                        {
                            while let _serde::__private::Some(__key) =
                                _serde::de::MapAccess::next_key::<__Field>(&mut __map)?
                            {
                                match __key {
                                    _ => {
                                        let _ = _serde::de::MapAccess::next_value::<
                                            _serde::de::IgnoredAny,
                                        >(
                                            &mut __map
                                        )?;
                                    }
                                }
                            }
                            _serde::__private::Ok(Config {})
                        }
                    }
                    #[doc(hidden)]
                    const FIELDS: &'static [&'static str] = &[];
                    _serde::Deserializer::deserialize_struct(
                        __deserializer,
                        "Config",
                        FIELDS,
                        __Visitor {
                            marker: _serde::__private::PhantomData::<Config>,
                            lifetime: _serde::__private::PhantomData,
                        },
                    )
                }
            }
        };
        #[allow(clippy::exhaustive_structs)]
        #[automatically_derived]
        impl ::core::marker::StructuralPartialEq for Config {}
        #[automatically_derived]
        #[allow(clippy::exhaustive_structs)]
        impl ::core::cmp::PartialEq for Config {
            #[inline]
            fn eq(&self, other: &Config) -> bool {
                true
            }
        }
        pub struct Outputs {
            channel: wasmrs_rx::FluxChannel<wasmrs::RawPayload, wasmrs::PayloadError>,
            #[allow(unused)]
            pub(crate) output: wick_packet::OutgoingPort<String>,
        }
        impl wick_component::Broadcast for Outputs {
            fn outputs_mut(&mut self) -> wick_packet::OutputIterator<'_> {
                wick_packet::OutputIterator::new(<[_]>::into_vec(
                    #[rustc_box]
                    ::alloc::boxed::Box::new([&mut self.output]),
                ))
            }
        }
        impl wick_packet::WasmRsChannel for Outputs {
            fn channel(&self) -> wasmrs_rx::FluxChannel<wasmrs::RawPayload, wasmrs::PayloadError> {
                self.channel.clone()
            }
        }
        impl wick_component::SingleOutput for Outputs {
            fn single_output(&mut self) -> &mut dyn wick_packet::Port {
                &mut self.output
            }
        }
        impl Outputs {
            pub fn new() -> (
                Self,
                wasmrs_rx::FluxReceiver<wasmrs::RawPayload, wasmrs::PayloadError>,
            ) {
                let (channel, rx) = wasmrs_rx::FluxChannel::new_parts();
                (
                    Self {
                        output: wick_packet::OutgoingPort::new("output", channel.clone()),
                        channel,
                    },
                    rx,
                )
            }
            pub fn with_channel(
                channel: wasmrs_rx::FluxChannel<wasmrs::RawPayload, wasmrs::PayloadError>,
            ) -> Self {
                Self {
                    output: wick_packet::OutgoingPort::new("output", channel.clone()),
                    channel,
                }
            }
        }
        #[cfg(not(target_family = "wasm"))]
        pub trait Operation {
            type Error: Send;
            type Outputs: Send;
            type Config: std::fmt::Debug + Send;
            #[allow(unused)]
            #[must_use]
            #[allow(clippy::type_complexity, clippy::type_repetition_in_bounds)]
            fn generate<'async_trait>(
                prompt: WickStream<wick_component::wick_packet::Packet>,
                outputs: Self::Outputs,
                ctx: wick_component::flow_component::Context<Self::Config>,
            ) -> ::core::pin::Pin<
                Box<
                    dyn ::core::future::Future<Output = std::result::Result<(), Self::Error>>
                        + ::core::marker::Send
                        + 'async_trait,
                >,
            >
            where
                Self: 'async_trait;
        }
    }
    ///The struct that the component implementation hinges around
    pub struct Component;
    #[automatically_derived]
    impl ::core::default::Default for Component {
        #[inline]
        fn default() -> Component {
            Component {}
        }
    }
    #[automatically_derived]
    impl ::core::clone::Clone for Component {
        #[inline]
        fn clone(&self) -> Component {
            Component
        }
    }
    impl Component {
        fn generate_wrapper(
            mut input: wasmrs_rx::BoxFlux<wasmrs::Payload, wasmrs::PayloadError>,
        ) -> std::result::Result<
            wasmrs_rx::BoxFlux<wasmrs::RawPayload, wasmrs::PayloadError>,
            wick_component::BoxError,
        > {
            let (channel, rx) =
                wasmrs_rx::FluxChannel::<wasmrs::RawPayload, wasmrs::PayloadError>::new_parts();
            let outputs = generate::Outputs::with_channel(channel.clone());
            runtime::spawn("generate_wrapper", async move {
                #[allow(unused_parens)]
                let (config, (prompt)) = {
                    struct Channels {
                        prompt: ::wick_component::wasmrs_rx::FluxChannel<
                            wick_component::wick_packet::Packet,
                            wick_component::AnyError,
                        >,
                    }
                    impl Channels {
                        #[allow(
                            clippy::missing_const_for_fn,
                            unreachable_pub,
                            unused,
                            unused_parens
                        )]
                        pub fn receivers(
                            &self,
                        ) -> Option<
                            (::wick_component::WickStream<wick_component::wick_packet::Packet>),
                        > {
                            Some((Box::pin(self.prompt.take_rx().ok()?)))
                        }
                    }
                    impl Default for Channels {
                        fn default() -> Self {
                            Self {
                                prompt: ::wick_component::wasmrs_rx::FluxChannel::new(),
                            }
                        }
                    }
                    #[allow(unused)]
                    let channels = Channels::default();
                    let (config_tx, config_rx) = ::wick_component::runtime::oneshot();
                    let mut config_tx = Some(config_tx);
                    let config_mono = Box::pin(config_rx);
                    let output_streams = (config_mono, channels.receivers().unwrap());
                    ::wick_component::runtime::spawn("payload_fan_out", async move {
                        #[allow(unused)]
                        use ::wick_component::StreamExt;
                        loop {
                            if let Some(Ok(payload)) = input.next().await {
                                let packet = {
                                    let mut packet: ::wick_component::wick_packet::Packet =
                                        payload.into();
                                    if let Some(config_tx) = config_tx.take() {
                                        if let Some(context) = packet.context() {
                                            let config: Result<
                                                        ::wick_component::wick_packet::ContextTransport<
                                                            generate::Config,
                                                        >,
                                                        _,
                                                    > = ::wick_component::wasmrs_codec::messagepack::deserialize(
                                                            &context,
                                                        )
                                                        .map_err(|e| {
                                                            let res = ::alloc::fmt::format(
                                                                format_args!("Cound not deserialize context: {0}", e),
                                                            );
                                                            res
                                                        });
                                            let _ = config_tx.send(config.map(
                                                ::wick_component::flow_component::Context::from,
                                            ));
                                        } else {
                                            packet = ::wick_component::wick_packet::Packet::component_error(
                                                        "No context attached to first invocation packet",
                                                    );
                                        }
                                    }
                                    packet
                                };
                                match packet.port() {
                                    "prompt" => {
                                        let tx = &channels.prompt;
                                        {
                                            use ::wick_component::wasmrs_rx::Observer;
                                            if packet.is_done() {
                                                tx.complete();
                                            } else {
                                                let _ = tx.send(packet);
                                            }
                                        }
                                    }
                                    ::wick_component::wick_packet::Packet::FATAL_ERROR => {
                                        use ::wick_component::wasmrs_rx::Observer;
                                        let error = packet.unwrap_err();
                                        channels
                                            .prompt
                                            .send_result(Err(::anyhow::__private::must_use({
                                                use ::anyhow::__private::kind::*;
                                                let error = match error.clone() {
                                                    error => (&error).anyhow_kind().new(error),
                                                };
                                                error
                                            })))
                                            .unwrap();
                                    }
                                    _ => {}
                                };
                            } else {
                                break;
                            }
                        }
                    });
                    output_streams
                };
                let config = match config.await {
                    Ok(Ok(config)) => config,
                    Err(e) => {
                        let _ = channel.send_result(
                            wick_packet::Packet::component_error({
                                let res = ::alloc::fmt::format(format_args!(
                                    "Component sent invalid context: {0}",
                                    e
                                ));
                                res
                            })
                            .into(),
                        );
                        return;
                    }
                    Ok(Err(e)) => {
                        let _ = channel.send_result(
                            wick_packet::Packet::component_error({
                                let res = ::alloc::fmt::format(format_args!(
                                    "Component sent invalid context: {0}",
                                    e
                                ));
                                res
                            })
                            .into(),
                        );
                        return;
                    }
                };
                use generate::Operation;
                if let Err(e) = Component::generate(Box::pin(prompt), outputs, config).await {
                    let _ = channel
                        .send_result(wick_packet::Packet::component_error(e.to_string()).into());
                }
            });
            Ok(Box::pin(rx))
        }
    }
}
use self::inference::Args;
use anyhow::Result;
use wick::*;
impl generate::Operation for Component {
    type Error = wick_component::AnyError;
    type Outputs = generate::Outputs;
    type Config = generate::Config;
    #[allow(
        clippy::async_yields_async,
        clippy::diverging_sub_expression,
        clippy::let_unit_value,
        clippy::no_effect_underscore_binding,
        clippy::shadow_same,
        clippy::type_complexity,
        clippy::type_repetition_in_bounds,
        clippy::used_underscore_binding
    )]
    fn generate<'async_trait>(
        input: WickStream<Packet>,
        outputs: Self::Outputs,
        ctx: Context<Self::Config>,
    ) -> ::core::pin::Pin<
        Box<
            dyn ::core::future::Future<Output = Result<(), Self::Error>>
                + ::core::marker::Send
                + 'async_trait,
        >,
    >
    where
        Self: 'async_trait,
    {
        Box::pin(async move {
            if let ::core::option::Option::Some(__ret) =
                ::core::option::Option::None::<Result<(), Self::Error>>
            {
                return __ret;
            }
            let input = input;
            let mut outputs = outputs;
            let ctx = ctx;
            let __ret: Result<(), Self::Error> = {
                let output_factory = || generate::Outputs::new();
                wick_component::unary::with_outputs(
                    input,
                    outputs,
                    output_factory,
                    &ctx,
                    &generate_wrapper,
                )
                .await?;
                Ok(())
            };
            #[allow(unreachable_code)]
            __ret
        })
    }
}
fn generate_wrapper(
    prompt: String,
    outputs: generate::Outputs,
    ctx: Context<generate::Config>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + 'static>> {
    Box::pin(async move { generate(prompt, outputs, ctx) })
}
fn generate(
    prompt: String,
    outputs: generate::Outputs,
    ctx: Context<generate::Config>,
) -> Result<()> {
    {
        let model = inference::load_model(&ctx.root_config().model)?;
        let tokenizer = inference::load_tokenizer(&ctx.root_config().tokenizer)?;
        let args = Args {
            prompt,
            temperature: None,
            top_p: None,
            repeat_penalty: 1.1,
            repeat_last_n: 64,
        };
        inference::run_inference(model, tokenizer, args);
        Ok(())
    }
}
