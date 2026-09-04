use bullet_lib::{
    game::{inputs::Chess768, outputs::MaterialCount},
    nn::optimiser::AdamW,
    trainer::{
        save::SavedFormat,
        schedule::{TrainingSchedule, TrainingSteps, lr, wdl},
        settings::LocalSettings,
    },
    value::{ValueTrainerBuilder, loader},
};

/// Feature-transformer width per perspective. Must match `HIDDEN_SIZE` in
/// the engine-side probe.
const HIDDEN_SIZE: usize = 1536;
/// Number of output buckets, selected by material count.
const OUTPUT_BUCKETS: usize = 8;
/// Eval scale. MUST equal the SCALE constant used by the engine at inference.
const SCALE: i32 = 400;
/// Feature-transformer quantisation ("QA").
const QA: i16 = 255;
/// Output-layer quantisation ("QB").
const QB: i16 = 64;

const DATA_PATHS: &[&str] = &[
    "data/S2/test77nov-unfilt-test79-maraprmay-v6-dd.skip-see-ge0.wdl-pdist.iter-1.bullet.bin",
    "data/S2/test77nov-unfilt-test79-maraprmay-v6-dd.skip-see-ge0.wdl-pdist.iter-2.bullet.bin",
    "data/S2/test77nov-unfilt-test79-maraprmay-v6-dd.skip-see-ge0.wdl-pdist.iter-3.bullet.bin",
    "data/S2/test77nov-unfilt-test79-maraprmay-v6-dd.skip-see-ge0.wdl-pdist.iter-4.bullet.bin",
    "data/S2/test77nov-unfilt-test79-maraprmay-v6-dd.skip-see-ge0.wdl-pdist.iter-5.bullet.bin",
    "data/S2/test77nov-unfilt-test79-maraprmay-v6-dd.skip-see-ge0.wdl-pdist.iter-6.bullet.bin",
    "data/S2/test77nov-unfilt-test79-maraprmay-v6-dd.skip-see-ge0.wdl-pdist.iter-7.bullet.bin",
    "data/S2/test77nov-unfilt-test79-maraprmay-v6-dd.skip-see-ge0.wdl-pdist.iter-8.bullet.bin",
];

fn main() {
    let mut trainer = ValueTrainerBuilder::default()
        .dual_perspective()
        .optimiser(AdamW)
        .inputs(Chess768)
        .output_buckets(MaterialCount::<OUTPUT_BUCKETS>)
        // quantised.bin layout, in this exact field order:
        //   l0w (i16, QA), l0b (i16, QA),
        //   l1w (i16, QB) [transposed: 8 x 2*HL], l1b (i16, QA*QB) [8]
        .save_format(&[
            SavedFormat::id("l0w").round().quantise::<i16>(QA),
            SavedFormat::id("l0b").round().quantise::<i16>(QA),
            SavedFormat::id("l1w")
                .round()
                .transpose()
                .quantise::<i16>(QB),
            SavedFormat::id("l1b").round().quantise::<i16>(QA * QB),
        ])
        .loss_fn(|output, target| output.sigmoid().squared_error(target))
        // (768 -> HIDDEN_SIZE)x2 -> 8, bucket-selected
        .build(|builder, stm_inputs, ntm_inputs, buckets| {
            let l0 = builder.new_affine("l0", 768, HIDDEN_SIZE);
            let l1 = builder.new_affine("l1", 2 * HIDDEN_SIZE, OUTPUT_BUCKETS);

            let stm_hidden = l0.forward(stm_inputs).screlu();
            let ntm_hidden = l0.forward(ntm_inputs).screlu();
            let hidden_layer = stm_hidden.concat(ntm_hidden);
            l1.forward(hidden_layer).select(buckets)
        });

    let schedule = TrainingSchedule {
        net_id: "lucky-v4".to_string(),
        eval_scale: SCALE as f32,
        steps: TrainingSteps {
            batch_size: 16_384,
            batches_per_superbatch: 36624,
            start_superbatch: 14,
            end_superbatch: 20,
        },
        // 0.5 = weight game-result and eval equally; raise toward 0.75 later.
        wdl_scheduler: wdl::ConstantWDL { value: 0.5 },
        lr_scheduler: lr::StepLR {
            start: 0.001,
            gamma: 0.1,
            step: 18,
        },
        save_rate: 5,
    };

    let settings = LocalSettings {
        threads: 4,
        test_set: None,
        output_directory: "checkpoints",
        batch_queue_size: 64,
    };

    // Native bulletformat shard(s) -> the simplest loader (no filter needed).
    let data_loader = loader::DirectSequentialDataLoader::new(DATA_PATHS);

    trainer.load_from_checkpoint("checkpoints/lucky-v4-13");
    trainer.run(&schedule, &settings, &data_loader);
}
