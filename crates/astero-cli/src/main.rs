use astero_core::session::Session;

fn main() -> std::process::ExitCode {
    match run() {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            std::process::ExitCode::FAILURE
        }
    }
}
fn run() -> Result<(), String> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args
        .first()
        .is_some_and(|a| a == "first-entry" || a == "_first-entry-worker")
    {
        let worker = args[0] == "_first-entry-worker";
        return astero_cli::entry::first::run(args.into_iter().skip(1).collect(), worker);
    }
    if args.first().is_some_and(|a| a == "entry-readiness") {
        let r=astero_cli::entry::parse(args.into_iter().skip(1)).map_err(|e|format!("{e}; entry-readiness uses native-map arguments plus --stack-base --stack-bytes --tls-base --max-runtime-bytes"))?;
        print!("{}", astero_cli::entry::execute(&r)?);
        return Ok(());
    }
    if args.first().is_some_and(|a| a == "native-map") {
        let r = astero_cli::native::parse(args.into_iter().skip(1)).map_err(|e| {
            format!("{e}; native-map uses stage-image arguments plus --max-native-bytes")
        })?;
        print!("{}", astero_cli::native::execute(&r)?);
        return Ok(());
    }
    if args.first().is_some_and(|a| a == "stage-image") {
        let request = astero_cli::staging::parse(args.into_iter().skip(1)).map_err(|e| {
            format!("{e}; stage-image uses load-plan arguments plus --max-mapped-bytes")
        })?;
        print!("{}", astero_cli::staging::execute(&request)?);
        return Ok(());
    }
    if args.first().is_some_and(|a| a == "load-plan") {
        let request = astero_cli::load_plan::parse(args.into_iter().skip(1))
            .map_err(|e| format!("{e}\n{}", astero_cli::load_plan::USAGE))?;
        let plan =
            astero_cli::load_plan::execute(&request).map_err(|e| format!("Load plan: {e:?}"))?;
        let text = astero_cli::load_plan::render(&plan);
        if plan.readiness() == astero_core::input::load_plan::Readiness::Blocked {
            return Err(text);
        }
        print!("{text}");
        return Ok(());
    }
    if args.first().is_some_and(|a| a == "ps5-identity") {
        let request = astero_cli::ps5_identity::parse(args.into_iter().skip(1))
            .map_err(|e| format!("{e}\n{}", astero_cli::ps5_identity::USAGE))?;
        let lower = request.linkage;
        let mut selection = astero_cli::acquisition::Selection::new(lower.symbols.acquisition);
        selection.acquire();
        let acquired = astero_cli::acquisition::render(&selection);
        if matches!(selection.state(), astero_cli::acquisition::State::Failed(_)) {
            return Err(acquired);
        }
        let linkage = astero_cli::linkage_evidence::observe_acquired(
            &selection,
            astero_core::input::linkage_evidence::LinkageLimits {
                hash: lower.symbols.hash,
                symbols: lower.symbols.symbols,
                max_relocations: lower.max_relocations,
            },
        )
        .map_err(|e| format!("Linkage request: {e:?}"))?;
        let report = astero_core::input::ps5_identity::observe(
            std::sync::Arc::new(linkage),
            astero_core::input::ps5_identity::IdentityLimits {
                max_identity_records: request.max_identity_records,
            },
        );
        let text = format!(
            "Acquisition stage:\n{acquired}\n{}",
            astero_cli::ps5_identity::render(&report)
        );
        if matches!(
            report.outcome(),
            astero_core::input::ps5_identity::IdentityOutcome::Failed(_)
        ) {
            return Err(text);
        }
        print!("{text}");
        return Ok(());
    }
    if args.first().is_some_and(|a| a == "linkage-evidence") {
        let request = astero_cli::linkage_evidence::parse(args.into_iter().skip(1))
            .map_err(|e| format!("{e}\n{}", astero_cli::linkage_evidence::USAGE))?;
        let mut selection = astero_cli::acquisition::Selection::new(request.symbols.acquisition);
        selection.acquire();
        let acquired = astero_cli::acquisition::render(&selection);
        if matches!(selection.state(), astero_cli::acquisition::State::Failed(_)) {
            return Err(acquired);
        }
        let report = astero_cli::linkage_evidence::observe_acquired(
            &selection,
            astero_core::input::linkage_evidence::LinkageLimits {
                hash: request.symbols.hash,
                symbols: request.symbols.symbols,
                max_relocations: request.max_relocations,
            },
        )
        .map_err(|e| format!("Linkage evidence request: {e:?}"))?;
        let text = format!(
            "Acquisition stage:\n{acquired}\n{}",
            astero_cli::linkage_evidence::render(&report)
        );
        if matches!(
            report.outcome(),
            astero_core::input::linkage_evidence::LinkageOutcome::Failed(_)
        ) {
            return Err(text);
        }
        print!("{text}");
        return Ok(());
    }
    if args.first().is_some_and(|a| a == "classify-symbols") {
        let request = astero_cli::classification::parse(args.into_iter().skip(1))
            .map_err(|e| format!("{e}\n{}", astero_cli::classification::USAGE))?;
        let mut selection = astero_cli::acquisition::Selection::new(request.symbols.acquisition);
        selection.acquire();
        let acquired = astero_cli::acquisition::render(&selection);
        if matches!(selection.state(), astero_cli::acquisition::State::Failed(_)) {
            return Err(acquired);
        }
        let report = astero_cli::classification::classify_acquired(
            &selection,
            request.symbols.hash,
            request.symbols.symbols,
            request.limits,
        )
        .map_err(|e| format!("Classification request: {e:?}"))?;
        let text = format!(
            "Acquisition stage:\n{acquired}\n{}",
            astero_cli::classification::render(&report)
        );
        if matches!(
            report.outcome(),
            astero_core::input::classification::ClassificationOutcome::Failed(_)
        ) {
            return Err(text);
        }
        print!("{text}");
        return Ok(());
    }
    if args.first().is_some_and(|a| a == "symbols") {
        let request = astero_cli::symbols::parse(args.into_iter().skip(1))
            .map_err(|e| format!("{e}\n{}", astero_cli::symbols::USAGE))?;
        let mut selection = astero_cli::acquisition::Selection::new(request.acquisition);
        selection.acquire();
        let acquired = astero_cli::acquisition::render(&selection);
        if matches!(selection.state(), astero_cli::acquisition::State::Failed(_)) {
            return Err(acquired);
        }
        let report =
            astero_cli::symbols::observe_acquired(&selection, request.hash, request.symbols)
                .map_err(|e| format!("Symbol request: {e:?}"))?;
        let text = format!(
            "Acquisition stage:\n{acquired}\n{}",
            astero_cli::symbols::render(&report)
        );
        if matches!(
            report.outcome(),
            astero_core::input::symbols::SymbolOutcome::Failed(_)
        ) {
            return Err(text);
        }
        print!("{text}");
        return Ok(());
    }
    if args.first().is_some_and(|a| a == "hash-metadata") {
        let request = astero_cli::hash_metadata::parse(args.into_iter().skip(1))
            .map_err(|e| format!("{e}\n{}", astero_cli::hash_metadata::USAGE))?;
        let mut selection = astero_cli::acquisition::Selection::new(request.acquisition);
        selection.acquire();
        let acquired = astero_cli::acquisition::render(&selection);
        if matches!(selection.state(), astero_cli::acquisition::State::Failed(_)) {
            return Err(acquired);
        }
        let report = astero_cli::hash_metadata::observe_acquired(&selection, request.limits)
            .map_err(|e| format!("Hash request: {e:?}"))?;
        let text = format!(
            "Acquisition stage:\n{acquired}\n{}",
            astero_cli::hash_metadata::render(&report)
        );
        if matches!(
            report.outcome(),
            astero_core::input::hash_metadata::HashMetadataOutcome::Failed(_)
        ) {
            return Err(text);
        }
        print!("{text}");
        return Ok(());
    }
    if args.first().is_some_and(|a| a == "string-references") {
        let request = astero_cli::string_references::parse(args.into_iter().skip(1))
            .map_err(|e| format!("{e}\n{}", astero_cli::string_references::USAGE))?;
        let mut selection = astero_cli::acquisition::Selection::new(request.acquisition);
        selection.acquire();
        let acquired = astero_cli::acquisition::render(&selection);
        if matches!(selection.state(), astero_cli::acquisition::State::Failed(_)) {
            return Err(acquired);
        }
        let report = astero_cli::string_references::observe_acquired(&selection, request.limits)
            .map_err(|e| format!("String-reference request: {e:?}"))?;
        let text = format!(
            "Acquisition stage:\n{acquired}\n{}",
            astero_cli::string_references::render(&report)
        );
        if matches!(
            report.outcome(),
            astero_core::input::string_references::StringReferenceOutcome::Failed(_)
        ) {
            return Err(text);
        }
        print!("{text}");
        return Ok(());
    }
    if args.first().is_some_and(|a| a == "descriptors") {
        let request = astero_cli::descriptors::parse(args.into_iter().skip(1))
            .map_err(|e| format!("{e}\n{}", astero_cli::descriptors::USAGE))?;
        let mut selection = astero_cli::acquisition::Selection::new(request.acquisition);
        selection.acquire();
        let acquired = astero_cli::acquisition::render(&selection);
        if matches!(selection.state(), astero_cli::acquisition::State::Failed(_)) {
            return Err(acquired);
        }
        let report = astero_cli::descriptors::observe_acquired(&selection, request.limits)
            .map_err(|e| format!("Descriptor request: {e:?}"))?;
        let text = format!(
            "Acquisition stage:\n{acquired}\n{}",
            astero_cli::descriptors::render(&report)
        );
        if matches!(
            report.outcome(),
            astero_core::input::descriptors::DescriptorOutcome::Failed(_)
        ) {
            return Err(text);
        }
        print!("{text}");
        return Ok(());
    }
    if args.first().is_some_and(|a| a == "dynamic") {
        let request = astero_cli::dynamic::parse(args.into_iter().skip(1))
            .map_err(|e| format!("{e}\n{}", astero_cli::dynamic::USAGE))?;
        let mut selection = astero_cli::acquisition::Selection::new(request.acquisition);
        selection.acquire();
        let acquired = astero_cli::acquisition::render(&selection);
        if matches!(selection.state(), astero_cli::acquisition::State::Failed(_)) {
            return Err(acquired);
        }
        let report = astero_cli::dynamic::observe_acquired(&selection, request.limits)
            .map_err(|e| format!("Dynamic request: {e:?}"))?;
        let text = format!(
            "Acquisition stage:\n{acquired}\n{}",
            astero_cli::dynamic::render(&report)
        );
        if matches!(
            report.outcome(),
            astero_core::input::dynamic::DynamicOutcome::Failed(_)
        ) {
            return Err(text);
        }
        print!("{text}");
        return Ok(());
    }
    if args.first().is_some_and(|a| a == "inspect") {
        let request = astero_cli::inspection::parse(args.into_iter().skip(1))
            .map_err(|e| format!("{e}\n{}", astero_cli::inspection::USAGE))?;
        let mut selection = astero_cli::acquisition::Selection::new(request.acquisition);
        selection.acquire();
        let acquired = astero_cli::acquisition::render(&selection);
        if matches!(selection.state(), astero_cli::acquisition::State::Failed(_)) {
            return Err(acquired);
        }
        let report = astero_cli::inspection::inspect_acquired(&selection, request.limits)
            .map_err(|e| format!("Inspection request: {e:?}"))?;
        let text = format!(
            "Acquisition stage:\n{acquired}\n{}",
            astero_cli::inspection::render(&report)
        );
        if matches!(
            report.outcome(),
            astero_core::input::inspection::InspectionOutcome::Failed(_)
        ) {
            return Err(text);
        }
        print!("{text}");
        return Ok(());
    }
    if args.first().is_some_and(|a| a == "acquire") {
        let request = astero_cli::acquisition::parse(args.into_iter().skip(1))
            .map_err(|e| format!("{e}\n{}", astero_cli::acquisition::USAGE))?;
        let mut selection = astero_cli::acquisition::Selection::new(request);
        selection.acquire();
        let text = astero_cli::acquisition::render(&selection);
        if matches!(selection.state(), astero_cli::acquisition::State::Failed(_)) {
            return Err(text);
        }
        print!("{text}");
        return Ok(());
    }
    if args.first().is_some_and(|a| a == "--linkage") {
        let synthetic = args.iter().skip(1).any(|a| a == "--synthetic");
        let details = args.iter().skip(1).any(|a| a == "--details");
        if args.len() != 1 + usize::from(synthetic) + usize::from(details)
            || args
                .iter()
                .skip(1)
                .any(|a| a != "--synthetic" && a != "--details")
        {
            return Err("Usage: astero-cli --linkage [--synthetic] [--details]".into());
        }
        let mut session = if synthetic {
            astero_cli::linkage::synthetic::session(8)?
        } else {
            Session::new().map_err(|e| format!("Create session: {e:?}"))?
        };
        session
            .initialize()
            .map_err(|e| format!("Initialize session: {e:?}"))?;
        let snapshot = astero_debug::snapshots::linkage::inspect_linkage(&session.observer())
            .map_err(|e| format!("Observe linkage: {e:?}"))?;
        if synthetic {
            println!("Synthetic in-memory ELF evidence demo; no loaded PS5 executable.");
        }
        print!("{}", astero_cli::linkage::render(&snapshot, details));
        return Ok(());
    }
    if !args.is_empty() {
        return Err(format!(
            "Usage: astero-cli first-entry <entry-readiness arguments> --wall-ms <1..500> --containment-ms <1000..30000> (REAL EXECUTION)\nUsage: astero-cli entry-readiness <native-map arguments> --stack-base <u64> --stack-bytes <u64> --tls-base <u64> --max-runtime-bytes <u64>\nUsage: astero-cli native-map <stage-image arguments> --max-native-bytes <u64>\nUsage: astero-cli stage-image <load-plan arguments> --max-mapped-bytes <u64>\nUsage: astero-cli load-plan <ps5-identity arguments> --image-bias <u64> --max-providers <u64> --max-plan-records <u64> [--provider <path>]\nUsage: astero-cli ps5-identity <linkage-evidence budgets> --max-identity-records <u64>\nUsage: astero-cli [--linkage [--synthetic] [--details]]\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}",
            astero_cli::acquisition::USAGE,
            astero_cli::inspection::USAGE,
            astero_cli::dynamic::USAGE,
            astero_cli::descriptors::USAGE,
            astero_cli::string_references::USAGE,
            astero_cli::hash_metadata::USAGE,
            astero_cli::symbols::USAGE,
            astero_cli::classification::USAGE
        ));
    }
    let mut session = Session::new().map_err(|e| format!("Create session: {e:?}"))?;
    session
        .initialize()
        .map_err(|e| format!("Initialize session: {e:?}"))?;
    let report = astero_debug::inspection::inspect_session(&session.observer())
        .map_err(|e| format!("Inspect session: {e:?}"))?;
    print!("{}", astero_cli::render(&report));
    Ok(())
}
