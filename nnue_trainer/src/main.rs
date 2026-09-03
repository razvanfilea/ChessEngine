//! v1 NNUE training run for lucky_chess.
//!
//! Architecture: `(768 -> HIDDEN_SIZE)x2 -> 1`, dual perspective, SCReLU.
//! This is deliberately the smallest useful net (see NNUE_TRAINING_PLAN.md) so
//! the whole train -> export -> probe -> SPRT loop can be validated before
//! adding output buckets, king buckets, etc.
//!
//! Run (AMD / RX 7800 XT):
//!   HSA_OVERRIDE_GFX_VERSION=11.0.0 \
//!     cargo run -r -p nnue_trainer --features rocm
//!
//! Output: checkpoints/lucky-v1-<superbatch>/{quantised.bin,raw.bin}
//! `quantised.bin` transmutes directly into the engine `Network` struct
//! documented at the bottom of this file.

use bullet_lib::{
    game::inputs::Chess768,
    nn::optimiser::AdamW,
    trainer::{
        save::SavedFormat,
        schedule::{lr, wdl, TrainingSchedule, TrainingSteps},
        settings::LocalSettings,
    },
    value::{loader, ValueTrainerBuilder},
};

/// Feature-transformer width per perspective. Must match `HIDDEN_SIZE` in
/// the engine-side probe.
const HIDDEN_SIZE: usize = 1024;
/// Eval scale. MUST equal the SCALE constant used by the engine at inference.
const SCALE: i32 = 400;
/// Feature-transformer quantisation ("QA").
const QA: i16 = 255;
/// Output-layer quantisation ("QB").
const QB: i16 = 64;

/// Decompressed linrock bulletformat shard. Get it with:
///   huggingface-cli download linrock/bullet-training-data \
///     "S2/test77nov-unfilt-test79-maraprmay-v6-dd.skip-see-ge0.wdl-pdist.iter-1.bullet.bin.zst" \
///     --repo-type dataset --local-dir ./data
///   zstd -d ./data/S2/*.iter-1.bullet.bin.zst
/// Then add more shard paths to the slice in `main` to scale up.
const DATA_PATH: &str =
    "data/S2/test77nov-unfilt-test79-maraprmay-v6-dd.skip-see-ge0.wdl-pdist.iter-1.bullet.bin";

fn main() {
    let mut trainer = ValueTrainerBuilder::default()
        // makes `ntm_inputs` available in the graph closure below
        .dual_perspective()
        // AdamW; default params clip weights to [-1.98, 1.98]
        .optimiser(AdamW)
        // 768 piece-square inputs (bullet extracts features itself)
        .inputs(Chess768)
        // quantised.bin layout, in this exact field order:
        //   l0w (i16, QA), l0b (i16, QA), l1w (i16, QB), l1b (i16, QA*QB)
        .save_format(&[
            SavedFormat::id("l0w").round().quantise::<i16>(QA),
            SavedFormat::id("l0b").round().quantise::<i16>(QA),
            SavedFormat::id("l1w").round().quantise::<i16>(QB),
            SavedFormat::id("l1b").round().quantise::<i16>(QA * QB),
        ])
        // target = wdl * game_result + (1 - wdl) * sigmoid(cp_score / SCALE)
        .loss_fn(|output, target| output.sigmoid().squared_error(target))
        // (768 -> HIDDEN_SIZE)x2 -> 1
        .build(|builder, stm_inputs, ntm_inputs| {
            let l0 = builder.new_affine("l0", 768, HIDDEN_SIZE);
            let l1 = builder.new_affine("l1", 2 * HIDDEN_SIZE, 1);

            let stm_hidden = l0.forward(stm_inputs).screlu();
            let ntm_hidden = l0.forward(ntm_inputs).screlu();
            let hidden_layer = stm_hidden.concat(ntm_hidden);
            l1.forward(hidden_layer)
        });

    let schedule = TrainingSchedule {
        net_id: "lucky-v2".to_string(),
        eval_scale: SCALE as f32,
        steps: TrainingSteps {
            batch_size: 16_384,
            // ~1 pass over the data per superbatch. Tune from the shard's
            // position count: (bytes of .bin / 32) / batch_size. The default
            // ~100M positions/superbatch is a fine starting point.
            batches_per_superbatch: 18312,
            start_superbatch: 1,
            end_superbatch: 13,
        },
        // 0.5 = weight game-result and eval equally; raise toward 0.75 later.
        wdl_scheduler: wdl::ConstantWDL { value: 0.5 },
        lr_scheduler: lr::StepLR { start: 0.001, gamma: 0.1, step: 18 },
        save_rate: 5,
    };

    let settings = LocalSettings {
        threads: 4,
        test_set: None,
        output_directory: "checkpoints",
        batch_queue_size: 64,
    };

    // Native bulletformat shard(s) -> the simplest loader (no filter needed).
    let data_loader = loader::DirectSequentialDataLoader::new(&[DATA_PATH]);

    trainer.run(&schedule, &settings, &data_loader);
}

// ===================== engine-side probe reference =====================
//
// The exported `quantised.bin` transmutes directly into this struct. Copy it
// into the engine's new `nnue2` module (adjust HIDDEN_SIZE to match). This is
// bullet's own reference inference (examples/simple.rs), reproduced so the
// engine and trainer definitions stay side by side.
//
// #[repr(C)]
// pub struct Network {
//     feature_weights: [Accumulator; 768], // col-major HIDDEN_SIZE x 768, QA
//     feature_bias:    Accumulator,        // QA
//     output_weights:  [i16; 2 * HIDDEN_SIZE], // QB
//     output_bias:     i16,                // QA * QB
// }
//
// #[repr(C, align(64))]
// pub struct Accumulator { vals: [i16; HIDDEN_SIZE] }
//
// fn screlu(x: i16) -> i32 { let y = i32::from(x).clamp(0, QA as i32); y * y }
//
// impl Network {
//     pub fn evaluate(&self, us: &Accumulator, them: &Accumulator) -> i32 {
//         let mut out = 0i32;
//         for (&i, &w) in us.vals.iter().zip(&self.output_weights[..HIDDEN_SIZE]) {
//             out += screlu(i) * w as i32;
//         }
//         for (&i, &w) in them.vals.iter().zip(&self.output_weights[HIDDEN_SIZE..]) {
//             out += screlu(i) * w as i32;
//         }
//         out /= QA as i32;
//         out += self.output_bias as i32;
//         out *= SCALE;
//         out /= (QA as i32) * (QB as i32);
//         out
//     }
// }
