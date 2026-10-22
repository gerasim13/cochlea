mod common;

use cochlea_features::{
    Audio, ProbeOpts, SegmentOpts, analyze_clipping, analyze_loudness, analyze_silence,
    count_silent_segments, detect_onsets, probe, segment_timeline,
};
use common::{click_track, sine_wave};

#[test]
fn focused_extractors_match_every_field_of_the_full_probe() {
    for sample_rate in [44_100, 48_000] {
        let mut partial = sine_wave(440.0, 0.4, 0.6, sample_rate);
        partial[..137].fill(0.0);
        partial.extend([0.0; 149]);
        for mono in [
            Vec::new(),
            vec![0.0; sample_rate as usize],
            partial,
            click_track(&[0.1, 0.3, 0.7], 1.0, sample_rate),
            sine_wave(440.0, 1.1, 0.6, sample_rate),
        ] {
            for channels in [1, 2] {
                let audio = Audio {
                    samples: mono
                        .iter()
                        .flat_map(|&x| std::iter::repeat_n(x, channels as usize))
                        .collect(),
                    channels,
                    sample_rate,
                };
                let opts = ProbeOpts::default();
                let full = probe(&audio, &opts);
                let loudness = analyze_loudness(&audio);
                assert_eq!(
                    serde_json::to_string(&loudness).unwrap(),
                    serde_json::to_string(&full.loudness).unwrap()
                );
                assert_eq!(
                    serde_json::to_string(&detect_onsets(&audio)).unwrap(),
                    serde_json::to_string(&full.onsets).unwrap()
                );
                assert_eq!(
                    serde_json::to_string(&analyze_clipping(&audio, loudness.true_peak_dbtp))
                        .unwrap(),
                    serde_json::to_string(&full.clipping).unwrap()
                );
                assert_eq!(
                    serde_json::to_string(&analyze_silence(
                        &audio.mono(),
                        sample_rate,
                        opts.silence_floor_dbfs
                    ))
                    .unwrap(),
                    serde_json::to_string(&full.silence).unwrap()
                );
            }
        }
    }
}

#[test]
fn silent_window_count_matches_timeline_boundaries_and_validation() {
    for sample_rate in [0, 44_100, 48_000] {
        for channels in [1, 2] {
            for samples in [
                Vec::new(),
                vec![0.0; 997],
                vec![0.001; 997],
                (0..997)
                    .map(|i| if i % 4 == 0 { 0.01 } else { 0.0 })
                    .collect(),
            ] {
                let audio = Audio {
                    samples,
                    channels,
                    sample_rate,
                };
                for window_ms in [0.5, 1.0, 5.0, 13.3, 1000.0, f64::NAN, f64::INFINITY] {
                    for floor in [-60.0, -20.0, f64::NAN] {
                        let opts = SegmentOpts::default()
                            .with_window_ms(window_ms)
                            .with_silence_floor_dbfs(floor);
                        let full = segment_timeline(&audio, &opts);
                        assert_eq!(
                            count_silent_segments(&audio, &opts),
                            full.segments.iter().filter(|s| s.silent).count()
                        );
                    }
                }
            }
        }
    }
}
