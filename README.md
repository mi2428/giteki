# giteki

`giteki` is a Rust CLI that queries Japan's Ministry of Internal Affairs and Communications [public Web API](https://www.tele.soumu.go.jp/j/sys/equ/tech/webapi/) for certified radio equipment and prints Giteki certification details in the terminal.

## Installation

### Build from Source

Install Rust and Cargo first, then build and install the binary with `make install`.

By default, the binary is installed to `~/.local/bin/giteki`.
Set `INSTALL_BINDIR` if you want to install it somewhere else.

```console
$ git clone https://github.com/mi2428/giteki
$ make -C giteki install
```

>[!TIP]
> Prebuilt binaries are also available from GitHub Releases for macOS and Linux, with amd64 and arm64 builds for each platform.
> Pick the asset that matches your machine, make it executable, and place it on your `PATH`.
> 
> ```console
> $ curl -L -o giteki https://github.com/mi2428/giteki/releases/download/vX.Y.Z/giteki-darwin-arm64
> ```

## Usage

```console
$ giteki --help

Display Giteki (技適) records using MIC equipment certification API (技術基準適合証明等機器検索API)

Usage: giteki [OPTIONS] [NUMBER]
       giteki <COMMAND>

Commands:
  file  Save an attachment PDF by file key (一覧詳細情報の添付ファイル取得キー)
  help  Print this message or the help of the given subcommand(s)

Arguments:
  [NUMBER]  Search by certification number (技術基準適合証明番号, 工事設計認証番号, or 届出番号)

Options:
  -n, --number <NUMBER>              Search by certification number (技術基準適合証明番号, 工事設計認証番号, or 届出番号)
      --name <NAME>                  Search by applicant name (氏名又は名称), partial match
  -t, --type-name <TYPE_NAME>        Search by model or type name (型式又は名称), partial match
      --organ-code <CODE>            Filter by certification body code (認証機関コード)
      --from <DATE>                  Start date (年月日), YYYYMMDD or YYYY-MM-DD
      --to <DATE>                    End date (年月日), YYYYMMDD or YYYY-MM-DD
      --radio-equipment-code <CODE>  Filter by specified radio equipment code (特定無線設備の種別コード)
      --tech-code <CODE>             Filter by certification type code (技術基準適合証明等の種類コード)
      --attachments                  Only search records with attachments (添付ファイル)
      --body-sar                     Only search Body SAR-supported records (BODYSAR対応)
  -l, --limit <LIMIT>                Maximum records to display. Fetches in API page-size units and truncates locally [default: 10]
      --offset <OFFSET>              Result offset [default: 0]
      --sort <SORT>                  API sort key [default: 1]
      --api-format <API_FORMAT>      API output format. csv/xml are printed as-is [default: json] [possible values: csv, json, xml]
      --json                         Print pretty JSON
  -h, --help                         Print help
  -V, --version                      Print version
```

```console
$ giteki 201-250579

API data last updated 2026-05-08

#1  201-250579  U7-Pro-XGS U7-Pro-XGS-B  Ubiquiti Inc.
───────────────────────────────────────────────────────────────────────────
種類                   相互承認(MRA)による工事設計認証
番号                   201-250579
年月日                 2025-08-14
氏名又は名称           Ubiquiti Inc.
型式又は名称           U7-Pro-XGS U7-Pro-XGS-B
特定無線設備の種別     第2条第19号に規定する特定無線設備
電波の型式             ・ G1D 2412~2472 MHz(13Ch)       6.40 mW/MHz
周波数及び空中線電力   ・ D1D, G1D 2412~2472 MHz(13Ch)  6.50 mW/MHz
                       ・ D1D, G1D 2412~2472 MHz(13Ch)  6.50 mW/MHz
                       ・ D1D, G1D 2422~2462 MHz(9Ch)   3.25 mW/MHz
                       ・ D1D, G1D 2412~2472 MHz(13Ch)  6.50 mW/MHz
                       ・ D1D, G1D 2422~2462 MHz(9Ch)   3.25 mW/MHz
                       ・ D1D, G1D 2412~2472 MHz(13Ch)  6.50 mW/MHz
                       ・ D1D, G1D 2422~2462 MHz(9Ch)   3.25 mW/MHz
スプリアス規定         新スプリアス規定
BODYSAR                ―
周波数維持機能         無
認証機関名称           Kiwa Nederland B.V.
添付ファイル名         201-250579_01_002.pdf
                       201-250579_01_003.pdf
                       201-250579_02_004.pdf
                       201-250579_02_005.pdf
                       201-250579_02_006.pdf
添付ファイルキー       201_N_1_250902N201_%E8%AA%8D%E8%A8%BC_35_*****_*****
添付ファイル数         外観写真等: 2 / 特性試験の結果: 3
───────────────────────────────────────────────────────────────────────────

#2  201-250579  U7-Pro-XGS U7-Pro-XGS-B  Ubiquiti Inc.
───────────────────────────────────────────────────────────────────────────
種類                   相互承認(MRA)による工事設計認証
番号                   201-250579
年月日                 2025-08-14
氏名又は名称           Ubiquiti Inc.
型式又は名称           U7-Pro-XGS U7-Pro-XGS-B
特定無線設備の種別     第2条第19号の3に規定する特定無線設備
電波の型式             ・ D1D, G1D 5180~5320 MHz(8Ch)    2.50 mW/MHz
周波数及び空中線電力   ・ D1D, G1D 5500~5720 MHz(12Ch)  10.00 mW/MHz
                       ・ D1D, G1D 5180~5320 MHz(8Ch)    2.50 mW/MHz
                       ・ D1D, G1D 5500~5720 MHz(12Ch)  10.00 mW/MHz
                       ・ D1D, G1D 5190~5310 MHz(4Ch)    1.30 mW/MHz
                       ・ D1D, G1D 5510~5710 MHz(6Ch)    4.90 mW/MHz
                       ・ D1D, G1D 5210~5290 MHz(2Ch)    0.60 mW/MHz
                       ・ D1D, G1D 5530~5690 MHz(3Ch)    2.50 mW/MHz
                       ・ D1D, G1D 5250~5250 MHz         0.31 mW/MHz
                       ・ D1D, G1D 5570~5570 MHz         1.20 mW/MHz
                       ・ D1D, G1D 5180~5320 MHz(8Ch)    2.50 mW/MHz
                       ・ D1D, G1D 5500~5720 MHz(12Ch)  10.00 mW/MHz
                       ・ D1D, G1D 5190~5310 MHz(4Ch)    1.30 mW/MHz
                       ・ D1D, G1D 5510~5710 MHz(6Ch)    4.90 mW/MHz
                       ・ D1D, G1D 5210~5290 MHz(2Ch)    0.60 mW/MHz
                       ・ D1D, G1D 5530~5690 MHz(3Ch)    2.50 mW/MHz
                       ・ D1D, G1D 5250~5250 MHz         0.31 mW/MHz
                       ・ D1D, G1D 5570~5570 MHz         1.20 mW/MHz
                       ・ D1D, G1D 5180~5320 MHz(8Ch)    2.50 mW/MHz
                       ・ D1D, G1D 5500~5720 MHz(12Ch)  10.00 mW/MHz
                       ・ D1D, G1D 5190~5310 MHz(4Ch)    1.30 mW/MHz
                       ・ D1D, G1D 5510~5710 MHz(6Ch)    4.90 mW/MHz
                       ・ D1D, G1D 5210~5290 MHz(2Ch)    0.60 mW/MHz
                       ・ D1D, G1D 5530~5690 MHz(3Ch)    2.50 mW/MHz
                       ・ D1D, G1D 5250~5250 MHz         0.31 mW/MHz
                       ・ D1D, G1D 5570~5570 MHz         1.20 mW/MHz
スプリアス規定         新スプリアス規定
BODYSAR                ―
周波数維持機能         無
認証機関名称           Kiwa Nederland B.V.
───────────────────────────────────────────────────────────────────────────

#3  201-250579  U7-Pro-XGS U7-Pro-XGS-B  Ubiquiti Inc.
───────────────────────────────────────────────────────────────────────────
種類                   相互承認(MRA)による工事設計認証
番号                   201-250579
年月日                 2025-08-14
氏名又は名称           Ubiquiti Inc.
型式又は名称           U7-Pro-XGS U7-Pro-XGS-B
特定無線設備の種別     第2条第80号に規定する特定無線設備
電波の型式             ・ D1D, G1D 5955~6415 MHz(24Ch)   2.50 mW/MHz
周波数及び空中線電力    ・ D1D, G1D 5955~6415 MHz(24Ch)   2.50 mW/MHz
                     ・ D1D, G1D 5965~6405 MHz(12Ch)   1.25 mW/MHz
                     ・ D1D, G1D 5985~6385 MHz(6Ch)   0.625 mW/MHz
                     ・ D1D, G1D 6025~6345 MHz(3Ch)   0.313 mW/MHz
                     ・ D1D, G1D 5955~6415 MHz(24Ch)   2.50 mW/MHz
                     ・ D1D, G1D 5965~6405 MHz(12Ch)   1.25 mW/MHz
                     ・ D1D, G1D 5985~6385 MHz(6Ch)   0.625 mW/MHz
                     ・ D1D, G1D 6025~6345 MHz(3Ch)   0.313 mW/MHz
                     ・ D1D, G1D 6105~6265 MHz(2Ch)    0.16 mW/MHz
スプリアス規定         新スプリアス規定
BODYSAR              ―
周波数維持機能         無
認証機関名称           Kiwa Nederland B.V.
───────────────────────────────────────────────────────────────────────────
```

### Attachment Files

The `file` subcommand saves an attachment PDF using the `attachmentFileKey` returned by the detail list API.
URL-encoded fragments in the key, such as `%E8...`, are decoded automatically by the CLI.

```console
$ giteki file '020_N_1_240830N020_%E8%AA%8D%E8%A8%BC_51_*****_*****' \
    --type 1 --number 1 --output attachment.pdf
```

```console
$ giteki file --help

Save an attachment PDF by file key (一覧詳細情報の添付ファイル取得キー)

Usage: giteki file [OPTIONS] --output <PATH> <AFK>

Arguments:
  <AFK>  Attachment file key (添付ファイル取得キー) returned by detail-list API (一覧詳細情報取得API)

Options:
      --type <AFT>     Attachment file type (添付ファイル種別). 1: 外観写真等, 2: 特性試験の結果
      --number <AFN>   Attachment file number (添付ファイル番号). Requires --type
  -o, --output <PATH>  Output PDF path
  -h, --help           Print help
```

## Development

`make release TAG=vX.Y.Z` builds four local release binaries, pushes the Git tag, creates or updates the GitHub Release with generated release notes, and uploads the release artifacts.
The default release matrix is macOS/Linux for amd64/arm64.
Before releasing, this repository must have a clean working tree.

```console
$ make

Development
  build             Build the host binary into bin/
  install           Build and install the host binary into INSTALL_BINDIR
  fmt               Format Rust sources. Use CHECK_ONLY=1 to check without writing
  lint              Run clippy with warnings treated as errors
  doc               Build rustdoc with warnings treated as errors
  test              Run unit tests
  check             Run formatting, lint, rustdoc, and tests
  clean             Remove local build artifacts

Distribution
  release           Build 4 local dist binaries, push the tag, and publish a GitHub release. Requires TAG=vX.Y.Z
  dist              Build release binaries into dist/. Use OS=darwin,linux and ARCH=amd64,arm64
  dist-smoke        Smoke-test Linux dist binaries in a Debian container
  checksums         Write SHA-256 checksums for dist artifacts

Help
  help              Show this help message

Variables:
  TAG               Release tag for make release, for example v0.1.0
  GIT_REMOTE        Release git remote, defaults to origin
  OS                Release OS list for make dist, defaults to darwin,linux
  ARCH              Release arch list for make dist, defaults to amd64,arm64
  INSTALL_BINDIR    Install directory, defaults to /Users/teo/.local/bin

Examples:
  make fmt CHECK_ONLY=1                         # Check formatting without writing
  make check                                    # Run local quality gates
  make dist OS=darwin,linux ARCH=amd64,arm64    # Build release binaries and checksums
  make release TAG=v0.1.0                       # Publish a GitHub release with local artifacts
```

## License

MIT
