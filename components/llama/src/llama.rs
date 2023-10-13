mod model;
mod weights;

use candle_core::{IndexOp, Tensor};
use candle_transformers::generation::LogitsProcessor;
use model::{Config, Llama};

use tokenizers::Tokenizer;
use weights::TransformerWeights;

use anyhow::Result;

use model::Cache;
use wick_component::{runtime::yield_now, FluxChannel, Observer};

thread_local! {
  static MODEL : std::cell::UnsafeCell<Option<Llama>> = std::cell::UnsafeCell::new(None);
  static TOKENIZER : std::cell::UnsafeCell<Option<Tokenizer>> = std::cell::UnsafeCell::new(None);
}

#[derive(Debug)]
pub(crate) struct Args {
    pub(crate) prompt: String,

    /// The temperature used to generate samples.
    pub(crate) temperature: Option<f64>,

    /// Nucleus sampling probability cutoff.
    pub(crate) top_p: Option<f64>,

    /// Penalty to be applied for repeating tokens, 1. means no penalty.
    pub(crate) repeat_penalty: f32,

    /// The context size to consider for the repeat penalty.
    pub(crate) repeat_last_n: usize,

    /// The maximum number of tokens to generate.
    pub(crate) max_seq: usize,
}

pub(super) fn load_model(file: &str) -> Result<Llama> {
    MODEL.with(|model| {
        let model = unsafe { &mut *model.get() };
        if model.is_none() {
            *model = Some(load_model_inner(file)?);
        }
        Ok(model.as_ref().unwrap().clone())
    })
}

fn load_model_inner(file: &str) -> Result<Llama> {
    println!("opening file: {}", file);
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
        }
        Ok(tokenizer.as_ref().unwrap().clone())
    })
}

fn load_tokenizer_inner(file: &str) -> Result<Tokenizer> {
    println!("opening file: {}", file);
    Tokenizer::from_file(file).map_err(|e| anyhow::anyhow!(e))
}

pub(crate) async fn generate(
    model: Llama,
    tokenizer: Tokenizer,
    args: Args,
    tx: FluxChannel<String, anyhow::Error>,
) -> Result<()> {
    let mut logits_processor = LogitsProcessor::new(299792458, args.temperature, args.top_p);
    let mut index_pos = 0;

    let mut tokens = tokenizer
        .encode(args.prompt.clone(), true)
        .map_err(anyhow::Error::msg)?
        .get_ids()
        .to_vec();
    let device = candle_core::Device::Cpu;

    for index in 0.. {
        if tokens.len() >= model.seq_len() {
            break;
        }
        if index >= args.max_seq {
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
        // Extracting the last token as a string is complicated, here we just apply some simple
        // heuristics as it seems to work well enough for this example. See the following for more
        // details:
        // https://github.com/huggingface/tokenizers/issues/1141#issuecomment-1562644141
        if let Some(text) = tokenizer.id_to_token(next_token) {
            let text = text.replace('▁', " ").replace("<0x0A>", "\n");
            tx.send(text)?;
        }
        yield_now().await;
    }

    Ok(())
}
