use bullet_lib::{
    game::{inputs::ChessBucketsMirrored, outputs::MaterialCount},
    nn::{
        InitSettings, Shape,
        optimiser::{AdamW, AdamWOptimiser},
    },
    trainer::save::SavedFormat,
    value::{ValueTrainer, ValueTrainerBuilder},
};
use chess_engine::nnue::network::{
    BUCKET_LAYOUT, HIDDEN_SIZE, INPUT_BUCKETS, OUTPUT_BUCKETS, QA, QB,
};

pub fn build_trainer()
-> ValueTrainer<AdamWOptimiser, ChessBucketsMirrored, MaterialCount<OUTPUT_BUCKETS>> {
    ValueTrainerBuilder::default()
        .dual_perspective()
        .optimiser(AdamW)
        .inputs(ChessBucketsMirrored::new(BUCKET_LAYOUT))
        .output_buckets(MaterialCount::<OUTPUT_BUCKETS>)
        .save_format(&[
            SavedFormat::id("l0w")
                .transform(|store, weights| {
                    let factorizer = store.get("l0f").values.f32().repeat(INPUT_BUCKETS);
                    weights
                        .into_iter()
                        .zip(factorizer)
                        .map(|(a, b)| a + b)
                        .collect()
                })
                .round()
                .quantise::<i16>(QA as i16),
            SavedFormat::id("l0b").round().quantise::<i16>(QA as i16),
            SavedFormat::id("l1w")
                .round()
                .transpose()
                .quantise::<i16>(QB as i16),
            SavedFormat::id("l1b")
                .round()
                .quantise::<i16>((QA * QB) as i16),
        ])
        .loss_fn(|output, target| output.sigmoid().squared_error(target))
        .build(|builder, stm_inputs, ntm_inputs, buckets| {
            let l0f =
                builder.new_weights("l0f", Shape::new(HIDDEN_SIZE, 768), InitSettings::Zeroed);
            let expanded_factorizer = l0f.repeat(INPUT_BUCKETS);

            let mut l0 = builder.new_affine("l0", 768 * INPUT_BUCKETS, HIDDEN_SIZE);
            l0.weights = l0.weights + expanded_factorizer;

            let l1 = builder.new_affine("l1", 2 * HIDDEN_SIZE, OUTPUT_BUCKETS);

            let stm_hidden = l0.forward(stm_inputs).screlu();
            let ntm_hidden = l0.forward(ntm_inputs).screlu();
            let hidden_layer = stm_hidden.concat(ntm_hidden);
            l1.forward(hidden_layer).select(buckets)
        })
}
