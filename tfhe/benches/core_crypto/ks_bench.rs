#[path = "../utilities.rs"]
mod utilities;

use crate::utilities::{
    filter_parameters, get_bench_type, init_parameters_set, throughput_num_threads, write_to_json,
    BenchmarkType, CryptoParametersRecord, DesiredBackend, DesiredNoiseDistribution, OperatorType,
    ParametersSet, PARAMETERS_SET,
};
use criterion::{black_box, Criterion, Throughput};
use rayon::prelude::*;
use serde::Serialize;
use std::env;
use tfhe::boolean::prelude::*;
use tfhe::core_crypto::prelude::*;
use tfhe::keycache::NamedParam;
use tfhe::shortint::parameters::current_params::{
    V1_1_PARAM_MESSAGE_1_CARRY_1_KS_PBS_GAUSSIAN_2M128,
    V1_1_PARAM_MESSAGE_2_CARRY_2_KS_PBS_GAUSSIAN_2M128,
    V1_1_PARAM_MESSAGE_3_CARRY_3_KS_PBS_GAUSSIAN_2M128, VEC_ALL_MULTI_BIT_PBS_PARAMETERS,
};
#[cfg(not(feature = "gpu"))]
use tfhe::shortint::parameters::current_params::{
    V1_1_PARAM_MESSAGE_4_CARRY_4_KS_PBS_GAUSSIAN_2M128,
    V1_1_PARAM_MULTI_BIT_GROUP_2_MESSAGE_1_CARRY_1_KS_PBS_GAUSSIAN_2M64,
    V1_1_PARAM_MULTI_BIT_GROUP_2_MESSAGE_2_CARRY_2_KS_PBS_GAUSSIAN_2M64,
    V1_1_PARAM_MULTI_BIT_GROUP_2_MESSAGE_3_CARRY_3_KS_PBS_GAUSSIAN_2M64,
    V1_1_PARAM_MULTI_BIT_GROUP_3_MESSAGE_1_CARRY_1_KS_PBS_GAUSSIAN_2M64,
    V1_1_PARAM_MULTI_BIT_GROUP_3_MESSAGE_2_CARRY_2_KS_PBS_GAUSSIAN_2M64,
    V1_1_PARAM_MULTI_BIT_GROUP_3_MESSAGE_3_CARRY_3_KS_PBS_GAUSSIAN_2M64,
};
use tfhe::shortint::parameters::v1_1::VEC_ALL_CLASSIC_PBS_PARAMETERS;
use tfhe::shortint::parameters::{
    COMP_PARAM_MESSAGE_2_CARRY_2_KS_PBS_TUNIFORM_2M128,
    PARAM_MESSAGE_2_CARRY_2_KS_PBS_TUNIFORM_2M128,
};
#[cfg(feature = "gpu")]
use tfhe::shortint::parameters::{
    PARAM_GPU_MULTI_BIT_GROUP_4_MESSAGE_1_CARRY_1_KS_PBS_GAUSSIAN_2M128,
    PARAM_GPU_MULTI_BIT_GROUP_4_MESSAGE_1_CARRY_1_KS_PBS_TUNIFORM_2M128,
    PARAM_GPU_MULTI_BIT_GROUP_4_MESSAGE_2_CARRY_2_KS_PBS_GAUSSIAN_2M128,
    PARAM_GPU_MULTI_BIT_GROUP_4_MESSAGE_2_CARRY_2_KS_PBS_TUNIFORM_2M128,
    PARAM_GPU_MULTI_BIT_GROUP_4_MESSAGE_3_CARRY_3_KS_PBS_GAUSSIAN_2M128,
    PARAM_GPU_MULTI_BIT_GROUP_4_MESSAGE_3_CARRY_3_KS_PBS_TUNIFORM_2M128,
};
use tfhe::shortint::prelude::*;
use tfhe::shortint::{MultiBitPBSParameters, PBSParameters};

#[cfg(not(feature = "gpu"))]
const SHORTINT_BENCH_PARAMS: [ClassicPBSParameters; 5] = [
    PARAM_MESSAGE_2_CARRY_2_KS_PBS_TUNIFORM_2M128,
    V1_1_PARAM_MESSAGE_1_CARRY_1_KS_PBS_GAUSSIAN_2M128,
    V1_1_PARAM_MESSAGE_2_CARRY_2_KS_PBS_GAUSSIAN_2M128,
    V1_1_PARAM_MESSAGE_3_CARRY_3_KS_PBS_GAUSSIAN_2M128,
    V1_1_PARAM_MESSAGE_4_CARRY_4_KS_PBS_GAUSSIAN_2M128,
];

#[cfg(feature = "gpu")]
const SHORTINT_BENCH_PARAMS: [ClassicPBSParameters; 4] = [
    PARAM_MESSAGE_2_CARRY_2_KS_PBS_TUNIFORM_2M128,
    V1_1_PARAM_MESSAGE_1_CARRY_1_KS_PBS_GAUSSIAN_2M128,
    V1_1_PARAM_MESSAGE_2_CARRY_2_KS_PBS_GAUSSIAN_2M128,
    V1_1_PARAM_MESSAGE_3_CARRY_3_KS_PBS_GAUSSIAN_2M128,
];

#[cfg(not(feature = "gpu"))]
const SHORTINT_MULTI_BIT_BENCH_PARAMS: [MultiBitPBSParameters; 6] = [
    V1_1_PARAM_MULTI_BIT_GROUP_2_MESSAGE_1_CARRY_1_KS_PBS_GAUSSIAN_2M64,
    V1_1_PARAM_MULTI_BIT_GROUP_2_MESSAGE_2_CARRY_2_KS_PBS_GAUSSIAN_2M64,
    V1_1_PARAM_MULTI_BIT_GROUP_2_MESSAGE_3_CARRY_3_KS_PBS_GAUSSIAN_2M64,
    V1_1_PARAM_MULTI_BIT_GROUP_3_MESSAGE_1_CARRY_1_KS_PBS_GAUSSIAN_2M64,
    V1_1_PARAM_MULTI_BIT_GROUP_3_MESSAGE_2_CARRY_2_KS_PBS_GAUSSIAN_2M64,
    V1_1_PARAM_MULTI_BIT_GROUP_3_MESSAGE_3_CARRY_3_KS_PBS_GAUSSIAN_2M64,
];

#[cfg(feature = "gpu")]
const SHORTINT_MULTI_BIT_BENCH_PARAMS: [MultiBitPBSParameters; 6] = [
    PARAM_GPU_MULTI_BIT_GROUP_4_MESSAGE_1_CARRY_1_KS_PBS_TUNIFORM_2M128,
    PARAM_GPU_MULTI_BIT_GROUP_4_MESSAGE_2_CARRY_2_KS_PBS_TUNIFORM_2M128,
    PARAM_GPU_MULTI_BIT_GROUP_4_MESSAGE_3_CARRY_3_KS_PBS_TUNIFORM_2M128,
    PARAM_GPU_MULTI_BIT_GROUP_4_MESSAGE_1_CARRY_1_KS_PBS_GAUSSIAN_2M128,
    PARAM_GPU_MULTI_BIT_GROUP_4_MESSAGE_2_CARRY_2_KS_PBS_GAUSSIAN_2M128,
    PARAM_GPU_MULTI_BIT_GROUP_4_MESSAGE_3_CARRY_3_KS_PBS_GAUSSIAN_2M128,
];

const BOOLEAN_BENCH_PARAMS: [(&str, BooleanParameters); 2] = [
    ("BOOLEAN_DEFAULT_PARAMS", DEFAULT_PARAMETERS),
    (
        "BOOLEAN_TFHE_LIB_PARAMS",
        PARAMETERS_ERROR_PROB_2_POW_MINUS_165,
    ),
];

fn benchmark_parameters_64bits() -> Vec<(String, CryptoParametersRecord<u64>)> {
    match PARAMETERS_SET.get().unwrap() {
        ParametersSet::Default => SHORTINT_BENCH_PARAMS
            .iter()
            .map(|params| {
                (
                    params.name(),
                    <ClassicPBSParameters as Into<PBSParameters>>::into(*params)
                        .to_owned()
                        .into(),
                )
            })
            .collect::<Vec<(String, CryptoParametersRecord<u64>)>>(),
        ParametersSet::All => {
            filter_parameters(
                &VEC_ALL_CLASSIC_PBS_PARAMETERS,
                DesiredNoiseDistribution::Both,
                DesiredBackend::Cpu, // No parameters set are specific to GPU in this vector
            )
            .into_iter()
            .map(|(params, name)| {
                (
                    name.to_string(),
                    <ClassicPBSParameters as Into<PBSParameters>>::into(*params)
                        .to_owned()
                        .into(),
                )
            })
            .collect::<Vec<(String, CryptoParametersRecord<u64>)>>()
        }
    }
}

fn multi_bit_benchmark_parameters_64bits() -> Vec<(String, CryptoParametersRecord<u64>)> {
    match PARAMETERS_SET.get().unwrap() {
        ParametersSet::Default => SHORTINT_MULTI_BIT_BENCH_PARAMS
            .iter()
            .map(|params| {
                (
                    params.name(),
                    <MultiBitPBSParameters as Into<PBSParameters>>::into(*params)
                        .to_owned()
                        .into(),
                )
            })
            .collect::<Vec<(String, CryptoParametersRecord<u64>)>>(),
        ParametersSet::All => {
            let desired_noise = DesiredNoiseDistribution::Both;
            let desired_backend = if cfg!(feature = "gpu") {
                DesiredBackend::Gpu
            } else {
                DesiredBackend::Cpu
            };

            filter_parameters(
                &VEC_ALL_MULTI_BIT_PBS_PARAMETERS,
                desired_noise,
                desired_backend,
            )
            .into_iter()
            .map(|(params, name)| {
                (
                    name.to_string(),
                    <MultiBitPBSParameters as Into<PBSParameters>>::into(*params)
                        .to_owned()
                        .into(),
                )
            })
            .collect()
        }
    }
}

fn benchmark_parameters_32bits() -> Vec<(String, CryptoParametersRecord<u32>)> {
    BOOLEAN_BENCH_PARAMS
        .iter()
        .map(|(name, params)| (name.to_string(), params.to_owned().into()))
        .collect()
}

fn benchmark_compression_parameters() -> Vec<(String, CryptoParametersRecord<u64>)> {
    vec![(
        COMP_PARAM_MESSAGE_2_CARRY_2_KS_PBS_TUNIFORM_2M128.name(),
        (
            COMP_PARAM_MESSAGE_2_CARRY_2_KS_PBS_TUNIFORM_2M128,
            PARAM_MESSAGE_2_CARRY_2_KS_PBS_TUNIFORM_2M128,
        )
            .into(),
    )]
}

fn keyswitch<Scalar: UnsignedTorus + CastInto<usize> + Serialize>(
    criterion: &mut Criterion,
    parameters: &[(String, CryptoParametersRecord<Scalar>)],
) {
    let bench_name = "core_crypto::keyswitch";
    let mut bench_group = criterion.benchmark_group(bench_name);

    // Create the PRNG
    let mut seeder = new_seeder();
    let seeder = seeder.as_mut();
    let mut encryption_generator =
        EncryptionRandomGenerator::<DefaultRandomGenerator>::new(seeder.seed(), seeder);
    let mut secret_generator = SecretRandomGenerator::<DefaultRandomGenerator>::new(seeder.seed());

    for (name, params) in parameters.iter() {
        let lwe_dimension = params.lwe_dimension.unwrap();
        let glwe_dimension = params.glwe_dimension.unwrap();
        let polynomial_size = params.polynomial_size.unwrap();
        let ks_decomp_base_log = params.ks_base_log.unwrap();
        let ks_decomp_level_count = params.ks_level.unwrap();

        let lwe_sk =
            allocate_and_generate_new_binary_lwe_secret_key(lwe_dimension, &mut secret_generator);

        let glwe_sk = allocate_and_generate_new_binary_glwe_secret_key(
            glwe_dimension,
            polynomial_size,
            &mut secret_generator,
        );
        let big_lwe_sk = glwe_sk.into_lwe_secret_key();
        let ksk_big_to_small = allocate_and_generate_new_lwe_keyswitch_key(
            &big_lwe_sk,
            &lwe_sk,
            ks_decomp_base_log,
            ks_decomp_level_count,
            params.lwe_noise_distribution.unwrap(),
            params.ciphertext_modulus.unwrap(),
            &mut encryption_generator,
        );

        let bench_id;

        match get_bench_type() {
            BenchmarkType::Latency => {
                let ct = allocate_and_encrypt_new_lwe_ciphertext(
                    &big_lwe_sk,
                    Plaintext(Scalar::ONE),
                    params.lwe_noise_distribution.unwrap(),
                    params.ciphertext_modulus.unwrap(),
                    &mut encryption_generator,
                );

                let mut output_ct = LweCiphertext::new(
                    Scalar::ZERO,
                    lwe_sk.lwe_dimension().to_lwe_size(),
                    params.ciphertext_modulus.unwrap(),
                );

                bench_id = format!("{bench_name}::{name}");
                {
                    bench_group.bench_function(&bench_id, |b| {
                        b.iter(|| {
                            keyswitch_lwe_ciphertext(&ksk_big_to_small, &ct, &mut output_ct);
                            black_box(&mut output_ct);
                        })
                    });
                }
            }
            BenchmarkType::Throughput => {
                bench_id = format!("{bench_name}::throughput::{name}");
                let blocks: usize = 1;
                let elements = throughput_num_threads(blocks, 1); // FIXME This number of element do not staturate the target machine
                bench_group.throughput(Throughput::Elements(elements));
                bench_group.bench_function(&bench_id, |b| {
                    let setup_encrypted_values = || {
                        let input_cts = (0..elements)
                            .map(|_| {
                                allocate_and_encrypt_new_lwe_ciphertext(
                                    &big_lwe_sk,
                                    Plaintext(Scalar::ONE),
                                    params.lwe_noise_distribution.unwrap(),
                                    params.ciphertext_modulus.unwrap(),
                                    &mut encryption_generator,
                                )
                            })
                            .collect::<Vec<_>>();

                        let output_cts = (0..elements)
                            .map(|_| {
                                LweCiphertext::new(
                                    Scalar::ZERO,
                                    lwe_sk.lwe_dimension().to_lwe_size(),
                                    params.ciphertext_modulus.unwrap(),
                                )
                            })
                            .collect::<Vec<_>>();

                        (input_cts, output_cts)
                    };

                    b.iter_batched(
                        setup_encrypted_values,
                        |(input_cts, mut output_cts)| {
                            input_cts
                                .par_iter()
                                .zip(output_cts.par_iter_mut())
                                .for_each(|(input_ct, output_ct)| {
                                    keyswitch_lwe_ciphertext(
                                        &ksk_big_to_small,
                                        input_ct,
                                        output_ct,
                                    );
                                })
                        },
                        criterion::BatchSize::SmallInput,
                    )
                });
            }
        };

        let bit_size = (params.message_modulus.unwrap_or(2) as u32).ilog2();
        write_to_json(
            &bench_id,
            *params,
            name,
            "ks",
            &OperatorType::Atomic,
            bit_size,
            vec![bit_size],
        );
    }
}

fn packing_keyswitch<Scalar, F>(
    criterion: &mut Criterion,
    bench_name: &str,
    parameters: &[(String, CryptoParametersRecord<Scalar>)],
    ks_op: F,
) where
    Scalar: UnsignedTorus + CastInto<usize> + Serialize,
    F: Fn(
            &LwePackingKeyswitchKey<Vec<Scalar>>,
            &LweCiphertextList<Vec<Scalar>>,
            &mut GlweCiphertext<Vec<Scalar>>,
        ) + Sync
        + Send,
{
    let bench_name = format!("core_crypto::{bench_name}");
    let mut bench_group = criterion.benchmark_group(&bench_name);

    // Create the PRNG
    let mut seeder = new_seeder();
    let seeder = seeder.as_mut();
    let mut encryption_generator =
        EncryptionRandomGenerator::<DefaultRandomGenerator>::new(seeder.seed(), seeder);
    let mut secret_generator = SecretRandomGenerator::<DefaultRandomGenerator>::new(seeder.seed());

    for (name, params) in parameters.iter() {
        let lwe_dimension = params.lwe_dimension.unwrap();
        let packing_glwe_dimension = params.packing_ks_glwe_dimension.unwrap();
        let packing_polynomial_size = params.packing_ks_polynomial_size.unwrap();
        let packing_ks_decomp_base_log = params.packing_ks_base_log.unwrap();
        let packing_ks_decomp_level_count = params.packing_ks_level.unwrap();
        let ciphertext_modulus = params.ciphertext_modulus.unwrap();
        let count = params.lwe_per_glwe.unwrap();

        let lwe_sk =
            allocate_and_generate_new_binary_lwe_secret_key(lwe_dimension, &mut secret_generator);

        let glwe_sk = allocate_and_generate_new_binary_glwe_secret_key(
            packing_glwe_dimension,
            packing_polynomial_size,
            &mut secret_generator,
        );

        let pksk = allocate_and_generate_new_lwe_packing_keyswitch_key(
            &lwe_sk,
            &glwe_sk,
            packing_ks_decomp_base_log,
            packing_ks_decomp_level_count,
            params.packing_ks_key_noise_distribution.unwrap(),
            ciphertext_modulus,
            &mut encryption_generator,
        );

        let bench_id;

        match get_bench_type() {
            BenchmarkType::Latency => {
                let mut input_lwe_list = LweCiphertextList::new(
                    Scalar::ZERO,
                    lwe_sk.lwe_dimension().to_lwe_size(),
                    count,
                    ciphertext_modulus,
                );

                let plaintext_list = PlaintextList::new(
                    Scalar::ZERO,
                    PlaintextCount(input_lwe_list.lwe_ciphertext_count().0),
                );

                encrypt_lwe_ciphertext_list(
                    &lwe_sk,
                    &mut input_lwe_list,
                    &plaintext_list,
                    params.lwe_noise_distribution.unwrap(),
                    &mut encryption_generator,
                );

                let mut output_glwe = GlweCiphertext::new(
                    Scalar::ZERO,
                    glwe_sk.glwe_dimension().to_glwe_size(),
                    glwe_sk.polynomial_size(),
                    ciphertext_modulus,
                );

                bench_id = format!("{bench_name}::{name}");
                {
                    bench_group.bench_function(&bench_id, |b| {
                        b.iter(|| {
                            ks_op(&pksk, &input_lwe_list, &mut output_glwe);
                            black_box(&mut output_glwe);
                        })
                    });
                }
            }
            BenchmarkType::Throughput => {
                bench_id = format!("{bench_name}::throughput::{name}");
                let blocks: usize = 1;
                let elements = throughput_num_threads(blocks, 1);
                bench_group.throughput(Throughput::Elements(elements));
                bench_group.bench_function(&bench_id, |b| {
                    let setup_encrypted_values = || {
                        let input_lwe_lists = (0..elements)
                            .map(|_| {
                                let mut input_lwe_list = LweCiphertextList::new(
                                    Scalar::ZERO,
                                    lwe_sk.lwe_dimension().to_lwe_size(),
                                    count,
                                    ciphertext_modulus,
                                );

                                let plaintext_list = PlaintextList::new(
                                    Scalar::ZERO,
                                    PlaintextCount(input_lwe_list.lwe_ciphertext_count().0),
                                );

                                encrypt_lwe_ciphertext_list(
                                    &lwe_sk,
                                    &mut input_lwe_list,
                                    &plaintext_list,
                                    params.lwe_noise_distribution.unwrap(),
                                    &mut encryption_generator,
                                );

                                input_lwe_list
                            })
                            .collect::<Vec<_>>();

                        let output_glwes = (0..elements)
                            .map(|_| {
                                GlweCiphertext::new(
                                    Scalar::ZERO,
                                    glwe_sk.glwe_dimension().to_glwe_size(),
                                    glwe_sk.polynomial_size(),
                                    ciphertext_modulus,
                                )
                            })
                            .collect::<Vec<_>>();

                        (input_lwe_lists, output_glwes)
                    };

                    b.iter_batched(
                        setup_encrypted_values,
                        |(input_lwe_lists, mut output_glwes)| {
                            input_lwe_lists
                                .par_iter()
                                .zip(output_glwes.par_iter_mut())
                                .for_each(|(input_lwe_list, output_glwe)| {
                                    ks_op(&pksk, input_lwe_list, output_glwe);
                                })
                        },
                        criterion::BatchSize::SmallInput,
                    )
                });
            }
        };

        let bit_size = (params.message_modulus.unwrap_or(2) as u32).ilog2();
        write_to_json(
            &bench_id,
            *params,
            name,
            "packing_ks",
            &OperatorType::Atomic,
            bit_size,
            vec![bit_size],
        );
    }
}

#[cfg(feature = "gpu")]
mod cuda {
    use crate::utilities::{
        cuda_local_keys_core, cuda_local_streams_core, get_bench_type, throughput_num_threads,
        write_to_json, BenchmarkType, CpuKeys, CpuKeysBuilder, CryptoParametersRecord, CudaIndexes,
        CudaLocalKeys, OperatorType,
    };
    use crate::{benchmark_parameters_64bits, multi_bit_benchmark_parameters_64bits};
    use criterion::{black_box, Criterion, Throughput};
    use rayon::prelude::*;
    use serde::Serialize;
    use tfhe::core_crypto::gpu::glwe_ciphertext_list::CudaGlweCiphertextList;
    use tfhe::core_crypto::gpu::lwe_ciphertext_list::CudaLweCiphertextList;
    use tfhe::core_crypto::gpu::{
        cuda_keyswitch_lwe_ciphertext, cuda_keyswitch_lwe_ciphertext_list_into_glwe_ciphertext,
        get_number_of_gpus, CudaStreams,
    };
    use tfhe::core_crypto::prelude::*;

    fn cuda_keyswitch<Scalar: UnsignedTorus + CastInto<usize> + CastFrom<u64> + Serialize>(
        criterion: &mut Criterion,
        parameters: &[(String, CryptoParametersRecord<Scalar>)],
    ) {
        let bench_name = "core_crypto::cuda::keyswitch";
        let mut bench_group = criterion.benchmark_group(bench_name);

        // Create the PRNG
        let mut seeder = new_seeder();
        let seeder = seeder.as_mut();
        let mut encryption_generator =
            EncryptionRandomGenerator::<DefaultRandomGenerator>::new(seeder.seed(), seeder);
        let mut secret_generator =
            SecretRandomGenerator::<DefaultRandomGenerator>::new(seeder.seed());

        for (name, params) in parameters.iter() {
            let lwe_dimension = params.lwe_dimension.unwrap();
            let glwe_dimension = params.glwe_dimension.unwrap();
            let polynomial_size = params.polynomial_size.unwrap();
            let ks_decomp_base_log = params.ks_base_log.unwrap();
            let ks_decomp_level_count = params.ks_level.unwrap();

            let lwe_sk = allocate_and_generate_new_binary_lwe_secret_key(
                lwe_dimension,
                &mut secret_generator,
            );

            let glwe_sk = allocate_and_generate_new_binary_glwe_secret_key(
                glwe_dimension,
                polynomial_size,
                &mut secret_generator,
            );
            let big_lwe_sk = glwe_sk.into_lwe_secret_key();
            let ksk_big_to_small = allocate_and_generate_new_lwe_keyswitch_key(
                &big_lwe_sk,
                &lwe_sk,
                ks_decomp_base_log,
                ks_decomp_level_count,
                params.lwe_noise_distribution.unwrap(),
                CiphertextModulus::new_native(),
                &mut encryption_generator,
            );

            let cpu_keys: CpuKeys<_> = CpuKeysBuilder::new()
                .keyswitch_key(ksk_big_to_small)
                .build();

            let bench_id;

            match get_bench_type() {
                BenchmarkType::Latency => {
                    let streams = CudaStreams::new_multi_gpu();
                    let gpu_keys = CudaLocalKeys::from_cpu_keys(&cpu_keys, None, &streams);

                    let ct = allocate_and_encrypt_new_lwe_ciphertext(
                        &big_lwe_sk,
                        Plaintext(Scalar::ONE),
                        params.lwe_noise_distribution.unwrap(),
                        CiphertextModulus::new_native(),
                        &mut encryption_generator,
                    );
                    let mut ct_gpu = CudaLweCiphertextList::from_lwe_ciphertext(&ct, &streams);

                    let output_ct = LweCiphertext::new(
                        Scalar::ZERO,
                        lwe_sk.lwe_dimension().to_lwe_size(),
                        CiphertextModulus::new_native(),
                    );
                    let mut output_ct_gpu =
                        CudaLweCiphertextList::from_lwe_ciphertext(&output_ct, &streams);

                    let h_indexes = [Scalar::ZERO];
                    let cuda_indexes = CudaIndexes::new(&h_indexes, &streams, 0);

                    bench_id = format!("{bench_name}::{name}");
                    {
                        bench_group.bench_function(&bench_id, |b| {
                            b.iter(|| {
                                cuda_keyswitch_lwe_ciphertext(
                                    gpu_keys.ksk.as_ref().unwrap(),
                                    &ct_gpu,
                                    &mut output_ct_gpu,
                                    &cuda_indexes.d_input,
                                    &cuda_indexes.d_output,
                                    &streams,
                                );
                                black_box(&mut ct_gpu);
                            })
                        });
                    }
                }
                BenchmarkType::Throughput => {
                    let gpu_keys_vec = cuda_local_keys_core(&cpu_keys, None);
                    let gpu_count = get_number_of_gpus() as usize;

                    bench_id = format!("{bench_name}::throughput::{name}");
                    let blocks: usize = 1;
                    let elements = throughput_num_threads(blocks, 1);
                    let elements_per_stream = elements as usize / gpu_count;
                    bench_group.throughput(Throughput::Elements(elements));
                    bench_group.sample_size(50);
                    bench_group.bench_function(&bench_id, |b| {
                        let setup_encrypted_values = || {
                            let local_streams = cuda_local_streams_core();

                            let plaintext_list = PlaintextList::new(
                                Scalar::ZERO,
                                PlaintextCount(elements_per_stream),
                            );

                            let input_cts = (0..gpu_count)
                                .map(|i| {
                                    let mut input_ct_list = LweCiphertextList::new(
                                        Scalar::ZERO,
                                        big_lwe_sk.lwe_dimension().to_lwe_size(),
                                        LweCiphertextCount(elements_per_stream),
                                        params.ciphertext_modulus.unwrap(),
                                    );
                                    encrypt_lwe_ciphertext_list(
                                        &big_lwe_sk,
                                        &mut input_ct_list,
                                        &plaintext_list,
                                        params.lwe_noise_distribution.unwrap(),
                                        &mut encryption_generator,
                                    );
                                    let input_ks_list = LweCiphertextList::from_container(
                                        input_ct_list.into_container(),
                                        big_lwe_sk.lwe_dimension().to_lwe_size(),
                                        params.ciphertext_modulus.unwrap(),
                                    );
                                    CudaLweCiphertextList::from_lwe_ciphertext_list(
                                        &input_ks_list,
                                        &local_streams[i],
                                    )
                                })
                                .collect::<Vec<_>>();

                            let output_cts = (0..gpu_count)
                                .map(|i| {
                                    let output_ct_list = LweCiphertextList::new(
                                        Scalar::ZERO,
                                        lwe_sk.lwe_dimension().to_lwe_size(),
                                        LweCiphertextCount(elements_per_stream),
                                        params.ciphertext_modulus.unwrap(),
                                    );
                                    CudaLweCiphertextList::from_lwe_ciphertext_list(
                                        &output_ct_list,
                                        &local_streams[i],
                                    )
                                })
                                .collect::<Vec<_>>();

                            let h_indexes = (0..(elements / gpu_count as u64))
                                .map(CastFrom::cast_from)
                                .collect::<Vec<_>>();
                            let cuda_indexes_vec = (0..gpu_count)
                                .map(|i| CudaIndexes::new(&h_indexes, &local_streams[i], 0))
                                .collect::<Vec<_>>();
                            local_streams.iter().for_each(|stream| stream.synchronize());

                            (input_cts, output_cts, cuda_indexes_vec, local_streams)
                        };

                        b.iter_batched(
                            setup_encrypted_values,
                            |(input_cts, mut output_cts, cuda_indexes_vec, local_streams)| {
                                (0..gpu_count)
                                    .into_par_iter()
                                    .zip(input_cts.par_iter())
                                    .zip(output_cts.par_iter_mut())
                                    .zip(local_streams.par_iter())
                                    .for_each(|(((i, input_ct), output_ct), local_stream)| {
                                        cuda_keyswitch_lwe_ciphertext(
                                            gpu_keys_vec[i].ksk.as_ref().unwrap(),
                                            input_ct,
                                            output_ct,
                                            &cuda_indexes_vec[i].d_input,
                                            &cuda_indexes_vec[i].d_output,
                                            local_stream,
                                        );
                                    })
                            },
                            criterion::BatchSize::SmallInput,
                        )
                    });
                }
            };

            let bit_size = (params.message_modulus.unwrap_or(2) as u32).ilog2();
            write_to_json(
                &bench_id,
                *params,
                name,
                "ks",
                &OperatorType::Atomic,
                bit_size,
                vec![bit_size],
            );
        }
    }

    fn cuda_packing_keyswitch<
        Scalar: UnsignedTorus + CastInto<usize> + CastFrom<u64> + Serialize,
    >(
        criterion: &mut Criterion,
        parameters: &[(String, CryptoParametersRecord<Scalar>)],
    ) {
        let bench_name = "core_crypto::cuda::packing_keyswitch";
        let mut bench_group = criterion.benchmark_group(bench_name);

        // Create the PRNG
        let mut seeder = new_seeder();
        let seeder = seeder.as_mut();
        let mut encryption_generator =
            EncryptionRandomGenerator::<DefaultRandomGenerator>::new(seeder.seed(), seeder);
        let mut secret_generator =
            SecretRandomGenerator::<DefaultRandomGenerator>::new(seeder.seed());

        for (name, params) in parameters.iter() {
            let lwe_dimension = params.lwe_dimension.unwrap();
            let glwe_dimension = params.glwe_dimension.unwrap();
            let polynomial_size = params.polynomial_size.unwrap();
            let ks_decomp_base_log = params.ks_base_log.unwrap();
            let ks_decomp_level_count = params.ks_level.unwrap();
            let glwe_noise_distribution = params.glwe_noise_distribution.unwrap();
            let ciphertext_modulus = params.ciphertext_modulus.unwrap();

            let lwe_sk = allocate_and_generate_new_binary_lwe_secret_key(
                lwe_dimension,
                &mut secret_generator,
            );

            let glwe_sk = allocate_and_generate_new_binary_glwe_secret_key(
                glwe_dimension,
                polynomial_size,
                &mut secret_generator,
            );

            let pksk = allocate_and_generate_new_lwe_packing_keyswitch_key(
                &lwe_sk,
                &glwe_sk,
                ks_decomp_base_log,
                ks_decomp_level_count,
                glwe_noise_distribution,
                ciphertext_modulus,
                &mut encryption_generator,
            );

            let cpu_keys: CpuKeys<_> = CpuKeysBuilder::new().packing_keyswitch_key(pksk).build();

            let bench_id;

            match get_bench_type() {
                BenchmarkType::Latency => {
                    let streams = CudaStreams::new_multi_gpu();
                    let gpu_keys = CudaLocalKeys::from_cpu_keys(&cpu_keys, None, &streams);

                    let mut input_ct_list = LweCiphertextList::new(
                        Scalar::ZERO,
                        lwe_sk.lwe_dimension().to_lwe_size(),
                        LweCiphertextCount(glwe_sk.polynomial_size().0),
                        ciphertext_modulus,
                    );

                    let plaintext_list = PlaintextList::new(
                        Scalar::ZERO,
                        PlaintextCount(input_ct_list.lwe_ciphertext_count().0),
                    );

                    encrypt_lwe_ciphertext_list(
                        &lwe_sk,
                        &mut input_ct_list,
                        &plaintext_list,
                        params.lwe_noise_distribution.unwrap(),
                        &mut encryption_generator,
                    );

                    let mut d_input_lwe_list =
                        CudaLweCiphertextList::from_lwe_ciphertext_list(&input_ct_list, &streams);

                    let mut d_output_glwe = CudaGlweCiphertextList::new(
                        glwe_sk.glwe_dimension(),
                        glwe_sk.polynomial_size(),
                        GlweCiphertextCount(1),
                        ciphertext_modulus,
                        &streams,
                    );

                    streams.synchronize();

                    bench_id = format!("{bench_name}::{name}");
                    {
                        bench_group.bench_function(&bench_id, |b| {
                            b.iter(|| {
                                cuda_keyswitch_lwe_ciphertext_list_into_glwe_ciphertext(
                                    gpu_keys.pksk.as_ref().unwrap(),
                                    &d_input_lwe_list,
                                    &mut d_output_glwe,
                                    &streams,
                                );
                                black_box(&mut d_input_lwe_list);
                            })
                        });
                    }
                }
                BenchmarkType::Throughput => {
                    let gpu_keys_vec = cuda_local_keys_core(&cpu_keys, None);
                    let gpu_count = get_number_of_gpus() as usize;

                    bench_id = format!("{bench_name}::throughput::{name}");
                    let blocks: usize = 1;
                    let elements = throughput_num_threads(blocks, 1);
                    let elements_per_stream = elements as usize / gpu_count;
                    bench_group.throughput(Throughput::Elements(elements));
                    bench_group.sample_size(50);
                    bench_group.bench_function(&bench_id, |b| {
                        let setup_encrypted_values = || {
                            let local_streams = cuda_local_streams_core();

                            let plaintext_list = PlaintextList::new(
                                Scalar::ZERO,
                                PlaintextCount(elements_per_stream),
                            );

                            let input_lwe_lists = (0..gpu_count)
                                .map(|i| {
                                    let mut input_ct_list = LweCiphertextList::new(
                                        Scalar::ZERO,
                                        lwe_sk.lwe_dimension().to_lwe_size(),
                                        LweCiphertextCount(glwe_sk.polynomial_size().0),
                                        ciphertext_modulus,
                                    );
                                    encrypt_lwe_ciphertext_list(
                                        &lwe_sk,
                                        &mut input_ct_list,
                                        &plaintext_list,
                                        params.lwe_noise_distribution.unwrap(),
                                        &mut encryption_generator,
                                    );

                                    CudaLweCiphertextList::from_lwe_ciphertext_list(
                                        &input_ct_list,
                                        &local_streams[i],
                                    )
                                })
                                .collect::<Vec<_>>();

                            let output_glwe_list = (0..gpu_count)
                                .map(|i| {
                                    CudaGlweCiphertextList::new(
                                        glwe_sk.glwe_dimension(),
                                        glwe_sk.polynomial_size(),
                                        GlweCiphertextCount(1),
                                        ciphertext_modulus,
                                        &local_streams[i],
                                    )
                                })
                                .collect::<Vec<_>>();

                            local_streams.iter().for_each(|stream| stream.synchronize());

                            (input_lwe_lists, output_glwe_list, local_streams)
                        };

                        b.iter_batched(
                            setup_encrypted_values,
                            |(input_lwe_lists, mut output_glwe_lists, local_streams)| {
                                (0..gpu_count)
                                    .into_par_iter()
                                    .zip(input_lwe_lists.par_iter())
                                    .zip(output_glwe_lists.par_iter_mut())
                                    .zip(local_streams.par_iter())
                                    .for_each(
                                        |(
                                            ((i, input_lwe_list), output_glwe_list),
                                            local_stream,
                                        )| {
                                            cuda_keyswitch_lwe_ciphertext_list_into_glwe_ciphertext(
                                                gpu_keys_vec[i].pksk.as_ref().unwrap(),
                                                input_lwe_list,
                                                output_glwe_list,
                                                local_stream,
                                            );
                                        },
                                    )
                            },
                            criterion::BatchSize::SmallInput,
                        )
                    });
                }
            };

            let bit_size = (params.message_modulus.unwrap_or(2) as u32).ilog2();
            write_to_json(
                &bench_id,
                *params,
                name,
                "packing_ks",
                &OperatorType::Atomic,
                bit_size,
                vec![bit_size],
            );
        }
    }

    pub fn cuda_ks_group() {
        let mut criterion: Criterion<_> =
            (Criterion::default().sample_size(2000)).configure_from_args();
        cuda_keyswitch(&mut criterion, &benchmark_parameters_64bits());
        cuda_packing_keyswitch(&mut criterion, &benchmark_parameters_64bits());
    }

    pub fn cuda_multi_bit_ks_group() {
        let mut criterion: Criterion<_> =
            (Criterion::default().sample_size(2000)).configure_from_args();
        cuda_keyswitch(&mut criterion, &multi_bit_benchmark_parameters_64bits());
        cuda_packing_keyswitch(&mut criterion, &multi_bit_benchmark_parameters_64bits());
    }
}

#[cfg(feature = "gpu")]
use cuda::{cuda_ks_group, cuda_multi_bit_ks_group};

pub fn ks_group() {
    let mut criterion: Criterion<_> = (Criterion::default()
        .sample_size(15)
        .measurement_time(std::time::Duration::from_secs(60)))
    .configure_from_args();
    keyswitch(&mut criterion, &benchmark_parameters_64bits());
    keyswitch(&mut criterion, &benchmark_parameters_32bits());
}

pub fn multi_bit_ks_group() {
    let mut criterion: Criterion<_> = (Criterion::default()
        .sample_size(15)
        .measurement_time(std::time::Duration::from_secs(60)))
    .configure_from_args();
    keyswitch(&mut criterion, &multi_bit_benchmark_parameters_64bits());
}

pub fn packing_ks_group() {
    let mut criterion: Criterion<_> = (Criterion::default()
        .sample_size(10)
        .measurement_time(std::time::Duration::from_secs(30)))
    .configure_from_args();
    packing_keyswitch(
        &mut criterion,
        "packing_keyswitch",
        &benchmark_compression_parameters(),
        keyswitch_lwe_ciphertext_list_and_pack_in_glwe_ciphertext,
    );
    packing_keyswitch(
        &mut criterion,
        "par_packing_keyswitch",
        &benchmark_compression_parameters(),
        par_keyswitch_lwe_ciphertext_list_and_pack_in_glwe_ciphertext,
    );
}

#[cfg(feature = "gpu")]
fn go_through_gpu_bench_groups(val: &str) {
    match val.to_lowercase().as_str() {
        "classical" => cuda_ks_group(),
        "multi_bit" => cuda_multi_bit_ks_group(),
        _ => panic!("unknown benchmark operations flavor"),
    };
}

#[cfg(not(feature = "gpu"))]
fn go_through_cpu_bench_groups(val: &str) {
    match val.to_lowercase().as_str() {
        "classical" => {
            ks_group();
            packing_ks_group()
        }
        "multi_bit" => multi_bit_ks_group(),
        _ => panic!("unknown benchmark operations flavor"),
    }
}

fn main() {
    init_parameters_set();

    match env::var("__TFHE_RS_PARAM_TYPE") {
        Ok(val) => {
            #[cfg(feature = "gpu")]
            go_through_gpu_bench_groups(&val);
            #[cfg(not(feature = "gpu"))]
            go_through_cpu_bench_groups(&val);
        }
        Err(_) => {
            ks_group();
            packing_ks_group()
        }
    };

    Criterion::default().configure_from_args().final_summary();
}
