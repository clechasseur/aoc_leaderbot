set windows-shell := ["powershell.exe", "-NoLogo", "-Command"]

toolchain := ""
tool := "cargo"

cargo := tool + (if toolchain != "" { " +" + toolchain } else { "" })

all_features := "true"
all_features_flag := if all_features == "true" { "--all-features" } else { "" }
feature_powerset_flag := if all_features == "true" { "--feature-powerset" } else { "" }

all_targets := "true"
all_targets_flag := if all_targets == "true" { "--all-targets" } else { "" }

message_format := ""
message_format_flag := if message_format != "" { "--message-format " + message_format } else { "" }

target_tuple := ""
target_tuple_flag := if target_tuple != "" { "--target " + target_tuple } else { "" }

release := "false"
release_flag := if release == "true" { "--release" } else { "" }

workspace := "true"
workspace_flag := if workspace == "true" { "--workspace" } else { "" }
all_flag := if workspace == "true" { "--all" } else { "" }

warnings_as_errors := "true"
clippy_flags := if warnings_as_errors == "true" { "-- -D warnings" } else { "" }

cargo_tarpaulin := tool + " tarpaulin"
cargo_hack := tool + " hack"

[private]
default:
    @just --list

# Run main executable
run *extra_args:
    {{cargo}} run {{all_features_flag}} {{target_tuple_flag}} {{release_flag}} {{ if extra_args != '' { '-- ' + extra_args } else { '' } }}

# Run an example
teach example_name *extra_args:
    {{cargo}} run {{all_features_flag}} {{target_tuple_flag}} {{release_flag}} --example {{example_name}} {{ if extra_args != '' { '-- ' + extra_args } else { '' } }}

# Run clippy and rustfmt on workspace files
tidy: clippy fmt

# Run clippy on workspace files
clippy *extra_args:
    {{cargo}} clippy {{workspace_flag}} {{all_targets_flag}} {{all_features_flag}} {{message_format_flag}} {{target_tuple_flag}} {{extra_args}} {{clippy_flags}}

# Run `cargo hack clippy` for the feature powerset
mega-clippy *extra_args:
    {{cargo_hack}} clippy {{workspace_flag}} {{all_targets_flag}} {{feature_powerset_flag}} {{message_format_flag}} {{target_tuple_flag}} {{extra_args}} {{clippy_flags}}

# Run rustfmt on workspace files
fmt *extra_args:
    cargo +nightly fmt {{all_flag}} {{message_format_flag}} {{extra_args}}

# Run `cargo check` on workspace
check *extra_args:
    {{cargo}} check {{workspace_flag}} {{all_targets_flag}} {{all_features_flag}} {{message_format_flag}} {{target_tuple_flag}} {{release_flag}} {{extra_args}}

# Run `cargo hack check` for the feature powerset
mega-check *extra_args:
    {{cargo_hack}} check {{workspace_flag}} --no-dev-deps --lib --bins {{feature_powerset_flag}} {{message_format_flag}} {{target_tuple_flag}} {{release_flag}} {{extra_args}}

# Run `cargo build` on workspace
build *extra_args:
    {{cargo}} build {{workspace_flag}} {{all_targets_flag}} {{all_features_flag}} {{message_format_flag}} {{target_tuple_flag}} {{release_flag}} {{extra_args}}

# Run `cargo test` on workspace
test *extra_args:
    {{cargo}} test {{workspace_flag}} {{all_features_flag}} {{message_format_flag}} {{target_tuple_flag}} {{release_flag}} {{extra_args}}

# Run `cargo update` to update dependencies in Cargo.lock
update *extra_args:
    {{cargo}} update {{extra_args}}

# Run `cargo tarpaulin` to produce code coverage
@tarpaulin *extra_args:
    @{{cargo_tarpaulin}} --target-dir target-tarpaulin {{extra_args}}
    {{ if env('CI', '') == '' { `just _open-tarpaulin` } else { ` ` } }}

[unix]
@_open-tarpaulin:
    open tarpaulin-report.html

[windows]
@_open-tarpaulin:
    ./tarpaulin-report.html

# Run `cargo llvm-cov` to produce code coverage
llvm-cov *extra_args:
    cargo +nightly llvm-cov --codecov --output-path codecov.json {{workspace_flag}} {{all_targets_flag}} {{all_features_flag}} {{target_tuple_flag}} {{extra_args}}
    cargo +nightly llvm-cov report --html {{ if env('CI', '') == '' { '--open' } else { '' } }}

# Generate documentation with rustdoc
doc package_name='': (_doc package_name)

_doc package_name='' $RUSTDOCFLAGS="-D warnings":
    {{cargo}} doc {{ if env('CI', '') != '' { '--no-deps' } else { '--open' } }} {{ if package_name != '' { '--package ' + package_name } else { workspace_flag } }} {{all_features_flag}} {{message_format_flag}}

# Check doc coverage with Nightly rustdoc
doc-coverage: _doc-coverage

_doc-coverage $RUSTDOCFLAGS="-Z unstable-options --show-coverage":
    cargo +nightly doc --no-deps {{workspace_flag}} {{all_features_flag}} {{message_format_flag}}

[private]
minimize:
    {{cargo}} hack --remove-dev-deps {{workspace_flag}}
    cargo +nightly update -Z minimal-versions

# Run `cargo minimal-versions check`
check-minimal package_name='': prep (_check-minimal-only package_name) && unprep

_check-minimal-only package_name='': (_rimraf "target/check-minimal-target")
    {{cargo}} minimal-versions check --target-dir target/check-minimal-target {{ if package_name != '' { '--package ' + package_name } else { workspace_flag } }} --lib --bins {{all_features_flag}} {{message_format_flag}}

# Run `cargo msrv` with `cargo minimal-versions check`
msrv-minimal package_name='': (prep "--manifest-backup-suffix .msrv-prep.outer.bak") && (unprep "--manifest-backup-suffix .msrv-prep.outer.bak")
    {{cargo}} msrv find -- just workspace="{{workspace}}" all_features="{{all_features}}" message_format="{{message_format}}" target_tuple="{{target_tuple}}" _check-minimal-only {{package_name}}

# Run `cargo msrv` with `cargo check`
msrv *extra_args: (prep "--manifest-backup-suffix .msrv-prep.outer.bak --no-merge-pinned-dependencies") && (unprep "--manifest-backup-suffix .msrv-prep.outer.bak")
    {{cargo}} msrv find -- just workspace="{{workspace}}" all_features="{{all_features}}" all_targets="{{all_targets}}" message_format="{{message_format}}" target_tuple="{{target_tuple}}" _msrv-check {{extra_args}}

_msrv-check *extra_args: (_rimraf "target/msrv-target")
    just workspace="{{workspace}}" all_features="{{all_features}}" all_targets="{{all_targets}}" message_format="{{message_format}}" target_tuple="{{target_tuple}}" check --target-dir target/msrv-target {{extra_args}}

# Perform `cargo publish` dry-run on a package
test-package package_name *extra_args:
    {{cargo}} publish --package {{package_name}} --dry-run {{extra_args}}

# Run `cargo msrv-prep` on workspace
prep *extra_args:
    {{cargo}} msrv-prep {{workspace_flag}} --backup-root-manifest {{extra_args}}

# Run `cargo msrv-unprep` on workspace
unprep *extra_args:
    {{cargo}} msrv-unprep {{workspace_flag}} --backup-root-manifest {{extra_args}}

# ----- Utilities -----

@_rimraf target_dir:
    {{ if path_exists(target_dir) == "true" { "just _rimraf-it '" + target_dir + "'" } else { "" } }}

[unix]
@_rimraf-it target_dir:
    rm -rf '{{target_dir}}'

[windows]
@_rimraf-it target_dir:
    Remove-Item "{{target_dir}}" -Recurse
