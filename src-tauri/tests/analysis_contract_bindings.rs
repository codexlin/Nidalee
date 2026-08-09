//! ts-rs 绑定导出验证（可单独运行）
//!
//! 目的：在不依赖 `cargo test --lib`（当前因既有 `#[cfg(test)]` 代码无法编译）的前提下，
//! 验证分析契约类型能真实导出到仓库既定目录 `src/types/generated/`，且生成内容的引用有效。
//!
//! 运行：`cargo test --test analysis_contract_bindings`

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use nidalee_lib::ai_contract::{AiInsight, AiPublicSettings};
use nidalee_lib::analysis_contract::{
    AnalysisCapabilities, AnalysisDegradationCode, AnalysisDepth, AnalysisDiagnostic, AnalysisFeature,
    AnalysisFeatureFlags, AnalysisMode, AnalysisPolicy, AnalysisQueueScope, MatchAnalysisRequest, MatchAnalysisResult,
};
use ts_rs::TS;

/// 仓库既定的前端生成类型目录（相对 src-tauri 包根，即测试进程 CWD）
const GENERATED_DIR: &str = "../src/types/generated";

/// 本任务契约必须产出的类型
const EXPECTED_TYPES: &[&str] = &[
    "AnalysisMode",
    "AnalysisDepth",
    "AnalysisFeatureFlags",
    "MatchAnalysisRequest",
    "AnalysisQueueScope",
    "AnalysisFeature",
    "AnalysisDegradationCode",
    "AnalysisDiagnostic",
    "AnalysisPolicy",
    "AnalysisCapabilities",
    "MatchAnalysisResult",
    "AiInsight",
    "AiInsightFinding",
    "AiInsightSuggestion",
    "AiPublicSettings",
];

/// Evidence 证据包必须产出的类型（`MatchAnalysisResult.evidence` 的依赖闭包）
const EXPECTED_EVIDENCE_TYPES: &[&str] = &[
    "EvidenceBundle",
    "MatchEvidence",
    "EvidenceQuality",
    "EvidencePosition",
    "EvidenceIssue",
    "EvidenceDiagnostic",
    "OpponentEvidence",
    "OpponentMatchMethod",
    "GamePhase",
    "PhaseEvidence",
    "PhaseOpponentDiff",
    "EventEvidence",
    "KeyEventEvidence",
    "EvidenceEventKind",
    "EventInvolvement",
    "DeathCause",
    "ActivityContext",
    "UnknownTimelineEvent",
    "TimelineSpan",
    "EvidenceSummary",
    "PhaseAverages",
    "EvidenceEventRates",
    "EvidenceConfidence",
    "ProcessInsight",
    "DeathBreakdownCard",
    "LaningProcessCard",
    "ObjectiveProcessCard",
    "VisionProcessCard",
    "ProcessAction",
    "ActivityBucketCount",
];

/// 生成目录的绝对路径
fn generated_dir() -> PathBuf {
    std::path::absolute(Path::new(GENERATED_DIR)).expect("解析生成目录绝对路径失败")
}

/// 导出契约类型及其全部依赖（同一进程内只执行一次）
fn ensure_exported() {
    static EXPORTED: OnceLock<()> = OnceLock::new();

    EXPORTED.get_or_init(|| {
        MatchAnalysisRequest::export_all().expect("导出 MatchAnalysisRequest 失败");
        MatchAnalysisResult::export_all().expect("导出 MatchAnalysisResult 失败");
        AiInsight::export_all().expect("导出 AiInsight 失败");
        AiPublicSettings::export_all().expect("导出 AiPublicSettings 失败");
    });
}

/// 断言某类型的 `export_to` 真实落点位于仓库生成目录
fn assert_output_path<T: TS + 'static + ?Sized>(type_name: &str) {
    let relative =
        <T as TS>::output_path().unwrap_or_else(|| panic!("{type_name} 缺少 #[ts(export_to = ...)]，无法导出"));

    let resolved = std::path::absolute(
        <T as TS>::default_output_path().unwrap_or_else(|| panic!("{type_name} 无法解析默认导出路径")),
    )
    .expect("解析默认导出路径失败");

    let expected = generated_dir().join(format!("{type_name}.ts"));

    assert_eq!(
        resolved,
        expected,
        "{type_name} 的 export_to（{}）落点错误：实际 {}，期望 {}",
        relative.display(),
        resolved.display(),
        expected.display()
    );
}

#[test]
fn test_export_to_paths_land_in_repo_generated_dir() {
    assert_output_path::<AnalysisMode>("AnalysisMode");
    assert_output_path::<AnalysisDepth>("AnalysisDepth");
    assert_output_path::<AnalysisFeatureFlags>("AnalysisFeatureFlags");
    assert_output_path::<MatchAnalysisRequest>("MatchAnalysisRequest");
    assert_output_path::<AnalysisQueueScope>("AnalysisQueueScope");
    assert_output_path::<AnalysisFeature>("AnalysisFeature");
    assert_output_path::<AnalysisDegradationCode>("AnalysisDegradationCode");
    assert_output_path::<AnalysisDiagnostic>("AnalysisDiagnostic");
    assert_output_path::<AnalysisPolicy>("AnalysisPolicy");
    assert_output_path::<AnalysisCapabilities>("AnalysisCapabilities");
    assert_output_path::<MatchAnalysisResult>("MatchAnalysisResult");
    assert_output_path::<AiInsight>("AiInsight");
    assert_output_path::<AiPublicSettings>("AiPublicSettings");

    let dir = generated_dir();
    assert!(
        dir.ends_with(Path::new("src").join("types").join("generated")),
        "生成目录必须是仓库既定的 src/types/generated，实际 {}",
        dir.display()
    );
}

#[test]
fn test_export_all_writes_expected_files() {
    ensure_exported();

    let dir = generated_dir();
    assert!(dir.is_dir(), "生成目录不存在：{}", dir.display());

    for type_name in EXPECTED_TYPES {
        let file = dir.join(format!("{type_name}.ts"));
        assert!(file.is_file(), "缺少生成文件：{}", file.display());

        let content = std::fs::read_to_string(&file).unwrap_or_else(|e| panic!("读取 {} 失败: {e}", file.display()));

        assert!(
            content.contains(&format!("type {type_name} =")),
            "{} 未声明类型 {type_name}",
            file.display()
        );

        // 队列 ID 必须导出为 number：Tauri 的 invoke 走 JSON，bigint 既发不出去也收不到
        assert!(
            !content.contains("bigint"),
            "{} 不应出现 bigint（JSON 无法承载），实际内容:\n{content}",
            file.display()
        );
    }
}

#[test]
fn test_generated_files_have_resolvable_imports() {
    ensure_exported();

    let dir = generated_dir();
    let mut missing: BTreeSet<String> = BTreeSet::new();

    // 校验整个依赖闭包：任一文件引用逃出生成目录，sync-types.mjs 合并后就会留下悬空类型
    for entry in std::fs::read_dir(&dir).expect("读取生成目录失败") {
        let file = entry.expect("遍历生成目录失败").path();
        if file.extension().and_then(|e| e.to_str()) != Some("ts") {
            continue;
        }

        let file_name = file
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default()
            .to_string();
        let content = std::fs::read_to_string(&file).unwrap_or_else(|e| panic!("读取 {} 失败: {e}", file.display()));

        for spec in import_specifiers(&content) {
            // sync-types.mjs 只合并生成目录内的文件并剥离 import，
            // 因此所有引用必须是同目录的 `./X` 且目标文件真实存在
            let Some(name) = spec.strip_prefix("./") else {
                missing.insert(format!("{file_name} -> {spec}（引用逃出生成目录）"));
                continue;
            };

            if !dir.join(format!("{name}.ts")).is_file() {
                missing.insert(format!("{file_name} -> {spec}（目标文件不存在）"));
            }
        }
    }

    assert!(missing.is_empty(), "生成文件存在无法解析的引用: {:?}", missing);
}

#[test]
fn test_result_exposes_typed_evidence_and_reserved_ai_slot() {
    ensure_exported();

    let content = std::fs::read_to_string(generated_dir().join("MatchAnalysisResult.ts"))
        .expect("读取 MatchAnalysisResult.ts 失败");

    assert!(
        content.contains("evidence?: EvidenceBundle"),
        "Evidence 必须是具体类型而不是 unknown，实际内容:\n{content}"
    );
    assert!(
        content.contains("import type { EvidenceBundle }"),
        "MatchAnalysisResult 应引用生成的 EvidenceBundle，实际内容:\n{content}"
    );
    assert!(
        content.contains("aiInsight?: AiInsight"),
        "AI 解读必须是具体 AiInsight 类型，实际内容:\n{content}"
    );
    assert!(
        content.contains("import type { AiInsight }"),
        "MatchAnalysisResult 应引用生成的 AiInsight，实际内容:\n{content}"
    );
}

#[test]
fn test_evidence_bundle_bindings_are_generated() {
    ensure_exported();

    let dir = generated_dir();
    for type_name in EXPECTED_EVIDENCE_TYPES {
        let file = dir.join(format!("{type_name}.ts"));
        assert!(file.is_file(), "缺少证据类型生成文件：{}", file.display());

        let content = std::fs::read_to_string(&file).unwrap_or_else(|e| panic!("读取 {} 失败: {e}", file.display()));
        assert!(
            content.contains(&format!("type {type_name} =")),
            "{} 未声明类型 {type_name}",
            file.display()
        );
        // gameId / queueId / 时间戳都是 i64/u64，必须显式导出为 number
        assert!(
            !content.contains("bigint"),
            "{} 不应出现 bigint（JSON 无法承载），实际内容:\n{content}",
            file.display()
        );
    }

    // 位置事实值必须是 ASCII 位置码，中文只属于前端展示层
    let position = std::fs::read_to_string(dir.join("EvidencePosition.ts")).expect("读取 EvidencePosition.ts 失败");
    for code in [
        "\"TOP\"",
        "\"JUNGLE\"",
        "\"MID\"",
        "\"ADC\"",
        "\"SUPPORT\"",
        "\"ARAM\"",
        "\"FLEX\"",
        "\"UNKNOWN\"",
    ] {
        assert!(position.contains(code), "EvidencePosition.ts 缺少 {code}:\n{position}");
    }
}

#[test]
fn test_request_exposes_camel_case_contract_fields() {
    ensure_exported();

    let content = std::fs::read_to_string(generated_dir().join("MatchAnalysisRequest.ts"))
        .expect("读取 MatchAnalysisRequest.ts 失败");

    for field in [
        "count",
        "mode",
        "depth",
        "queueId?",
        "queueIds?",
        "features",
        "maxAnalysisGames?",
        "perspective?",
    ] {
        assert!(
            content.contains(field),
            "MatchAnalysisRequest.ts 缺少字段 {field}，实际内容:\n{content}"
        );
    }
}

/// 提取 import 语句中的模块说明符（保留原始相对前缀）
fn import_specifiers(content: &str) -> Vec<String> {
    content
        .lines()
        .filter(|line| line.trim_start().starts_with("import"))
        .filter_map(|line| {
            let start = line.find("from \"")? + "from \"".len();
            let rest = &line[start..];
            let end = rest.find('"')?;
            Some(rest[..end].to_string())
        })
        .collect()
}
