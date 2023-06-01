# Changelog

## [2.3.1](https://github.com/equinor/septic-config-generator/compare/v2.3.0...v2.3.1) (2023-06-01)


### 🐛 Bug Fixes

* bitmask handles input value 0 ([#105](https://github.com/equinor/septic-config-generator/issues/105)) ([d0671cb](https://github.com/equinor/septic-config-generator/commit/d0671cbcfbecbdbbdd8077456c3ed8b9bbada41e))

## [2.3.0](https://github.com/equinor/septic-config-generator/compare/v2.2.1...v2.3.0) (2023-05-31)


### ✨ Features

* add bitmask() filter function ([#97](https://github.com/equinor/septic-config-generator/issues/97)) ([308550b](https://github.com/equinor/septic-config-generator/commit/308550b557d8757a481415f4cd30fee6296aa8b1))


### 🧹 Chores

* **deps:** bump chrono from 0.4.24 to 0.4.25 ([#99](https://github.com/equinor/septic-config-generator/issues/99)) ([83e90b5](https://github.com/equinor/septic-config-generator/commit/83e90b5605601e9a4635fe79f6ca218639885016))
* **deps:** bump clap from 4.1.8 to 4.3.0 ([#96](https://github.com/equinor/septic-config-generator/issues/96)) ([97e7de6](https://github.com/equinor/septic-config-generator/commit/97e7de6f072fd36aa4ed1bcb4408dfa7ce300e07))
* **deps:** bump regex from 1.7.1 to 1.8.3 ([#98](https://github.com/equinor/septic-config-generator/issues/98)) ([6a1db7a](https://github.com/equinor/septic-config-generator/commit/6a1db7aaf36b2d3e51bb8b5af74f017a4a5917c1))
* **deps:** bump serde_yaml from 0.9.19 to 0.9.21 ([#82](https://github.com/equinor/septic-config-generator/issues/82)) ([9ec4599](https://github.com/equinor/septic-config-generator/commit/9ec459929f0e508f94b394f291e5f6a91aac53ee))
* **deps:** bump tempfile from 3.4.0 to 3.5.0 ([#81](https://github.com/equinor/septic-config-generator/issues/81)) ([05930a0](https://github.com/equinor/septic-config-generator/commit/05930a08b4f4d479d9a75622a46c795402e285dc))


### 📚 Documentation

* update documentation for 2.3.0 ([#100](https://github.com/equinor/septic-config-generator/issues/100)) ([3669c7d](https://github.com/equinor/septic-config-generator/commit/3669c7df5cf770aa5e7e32a4411a747cba54f89f))

## [2.2.1](https://github.com/equinor/septic-config-generator/compare/v2.2.0...v2.2.1) (2023-04-25)


### 📚 Documentation

* update docs for v2.2 ([#70](https://github.com/equinor/septic-config-generator/issues/70)) ([483fc88](https://github.com/equinor/septic-config-generator/commit/483fc88892ce66f5d6add8ae871e8e26d0da6b26))


### 👷 CI/CD

* add security audit workflow ([#77](https://github.com/equinor/septic-config-generator/issues/77)) ([841677f](https://github.com/equinor/septic-config-generator/commit/841677f51f58b07258406f3ca16b876315325dfb))
* add test and coverage reporting ([#73](https://github.com/equinor/septic-config-generator/issues/73)) ([f79ba30](https://github.com/equinor/septic-config-generator/commit/f79ba305a5148daa74b42fa92f95c291939d86bd))
* improve workflows ([#79](https://github.com/equinor/septic-config-generator/issues/79)) ([8cfe092](https://github.com/equinor/septic-config-generator/commit/8cfe0922eab81510c1572efc33895fb565a8945e))


### 🧪 Tests

* make tests work in Windows environment ([#87](https://github.com/equinor/septic-config-generator/issues/87)) ([7679786](https://github.com/equinor/septic-config-generator/commit/76797866279f07d7a677af5e7765eec940ca972b))


### 🐛 Bug Fixes

* use crlf (\r\n) when adjusting spacing ([#88](https://github.com/equinor/septic-config-generator/issues/88)) ([b134fd0](https://github.com/equinor/septic-config-generator/commit/b134fd0104f32de4bb76d9b29598258feb16321e))

## [2.2.0](https://github.com/equinor/septic-config-generator/compare/v2.1.0...v2.2.0) (2023-03-28)


### 🧪 Tests

* add more tests ([b8d92c3](https://github.com/equinor/septic-config-generator/commit/b8d92c3c1048e7fce14fa6c58ade5e3d1bb2d10f))


### 👷 CI/CD

* chmod +x before tar and attach to release ([3222540](https://github.com/equinor/septic-config-generator/commit/3222540db2db3a28fa9216f5974a03bfdd34e558))
* don't perform cargo check ([b8d92c3](https://github.com/equinor/septic-config-generator/commit/b8d92c3c1048e7fce14fa6c58ade5e3d1bb2d10f))


### ✨ Features

* add arg to only build if input files changed ([#60](https://github.com/equinor/septic-config-generator/issues/60)) ([ad118bf](https://github.com/equinor/septic-config-generator/commit/ad118bf6044a5bca3b698bbdfbaefa1db841d908))


### 🐛 Bug Fixes

* ensure newline at EOF if adjustspacing=true ([b8d92c3](https://github.com/equinor/septic-config-generator/commit/b8d92c3c1048e7fce14fa6c58ade5e3d1bb2d10f))
* error msg when unvalid source 1st column ([#71](https://github.com/equinor/septic-config-generator/issues/71)) ([90dec90](https://github.com/equinor/septic-config-generator/commit/90dec9080d8992802e10edbc1fc05e7cbfafc4fc))


### 🧹 Chores

* **deps:** bump minijinja from 0.30.6 to 0.30.7 ([#58](https://github.com/equinor/septic-config-generator/issues/58)) ([dc74ce3](https://github.com/equinor/septic-config-generator/commit/dc74ce32766aae014e8d25358a0555d50c09c703))


### 🔨 Refactor

* clean up cmd_make ([b8d92c3](https://github.com/equinor/septic-config-generator/commit/b8d92c3c1048e7fce14fa6c58ade5e3d1bb2d10f))
* clean up confusing type names ([c485c7b](https://github.com/equinor/septic-config-generator/commit/c485c7bd4ac5d9ff3d36732ab7bedf757e6a2ed2))

## [2.1.0](https://github.com/equinor/septic-config-generator/compare/v2.0.0...v2.1.0) (2023-03-02)


### 📚 Documentation

* minor changes ([#41](https://github.com/equinor/septic-config-generator/issues/41)) ([9c3f8a4](https://github.com/equinor/septic-config-generator/commit/9c3f8a4a19f0a9c47a83aaa2547994505a85b053))


### ✨ Features

* add global variable {{ scgversion }} ([#43](https://github.com/equinor/septic-config-generator/issues/43)) ([14ba63a](https://github.com/equinor/septic-config-generator/commit/14ba63aef4b90ba1c30c5717ddd2cc9ec875b61e))
* add scg diff ([#44](https://github.com/equinor/septic-config-generator/issues/44)) ([d36f70e](https://github.com/equinor/septic-config-generator/commit/d36f70e7450c2c9d30f10387f573f99c24ec56f7))

## [2.0.0](https://github.com/equinor/septic-config-generator/compare/v1.0.0...v2.0.0) (2023-03-01)


### 📦 Build system

* rename binary to scg ([d083f24](https://github.com/equinor/septic-config-generator/commit/d083f2416d8ce6ae57a2c68927c9b806e2bb7624))


### 💎 Style

* disable colors ([1e95b0f](https://github.com/equinor/septic-config-generator/commit/1e95b0f525a12597d475875ee7c65c7f8b689a5f))


### 🧪 Tests

* add some test templates ([848f00a](https://github.com/equinor/septic-config-generator/commit/848f00ace4b03722f76d2e7bcf9e682fa10b7318))
* add test excel file ([e752533](https://github.com/equinor/septic-config-generator/commit/e7525339731420251a2f108db76e1ee23f4cf898))


### 🔨 Refactor

* add add_globals() ([e96a9ea](https://github.com/equinor/septic-config-generator/commit/e96a9eae24b44b0932967cf48e10f251a98e574d))
* add function to bubble minijinja errors ([2068737](https://github.com/equinor/septic-config-generator/commit/2068737e9966c86dfad06032423da94ab3e0e260))
* be more pedantic ([a8004a5](https://github.com/equinor/septic-config-generator/commit/a8004a58e344deec1cf4707549bd6e6b277f930e))
* create CtxDataType ([f2df1c1](https://github.com/equinor/septic-config-generator/commit/f2df1c19db9774becd1a2b5a49ec1f1bfd6b8e1f))
* extract closure into load_template() ([2c10c1c](https://github.com/equinor/septic-config-generator/commit/2c10c1c5a2414a0802a41e433f7d09ba7b6540cf))
* follow clippy advice ([a0f7875](https://github.com/equinor/septic-config-generator/commit/a0f78752263981be5cc29fbe6a6be0e3d2be5eb0))
* impl config read, simpler error handling ([0b62cb8](https://github.com/equinor/septic-config-generator/commit/0b62cb8082523aac3c1bca20d2025f04dada2f11))
* let renderer do the rendering ([9746c79](https://github.com/equinor/septic-config-generator/commit/9746c79a868a7809ab60c775cd1bc82e8909b2bd))
* make gitcommit a global variable ([e82c0d4](https://github.com/equinor/septic-config-generator/commit/e82c0d4fd54ef10918dd80b49a3880a039571d23))
* move jinja-stuff into separate module ([daed376](https://github.com/equinor/septic-config-generator/commit/daed3767d8aab791fe50a9963642f58d6f4fd485))
* move make command into own function ([f01ad94](https://github.com/equinor/septic-config-generator/commit/f01ad940c4d87dd3a0a34883bc123ca0fbf6fec2))
* move yaml parsing to lib.rs ([fec9164](https://github.com/equinor/septic-config-generator/commit/fec9164b70964fbea86b54dc9271efd6d214d73f))
* rename libs, separate 2 modules ([3308f31](https://github.com/equinor/septic-config-generator/commit/3308f31ddc2ed385f6a2a59be31144394da4ecac))
* split config read into own function ([5219c97](https://github.com/equinor/septic-config-generator/commit/5219c975c6d0886c05d1c2d69cf5fefca724d6ee))
* split excel read into own function ([ffc5905](https://github.com/equinor/septic-config-generator/commit/ffc59050e906b3e1c4b38fa421dce778a0ddf59d))
* split lib.rs into multiple files ([89bca89](https://github.com/equinor/septic-config-generator/commit/89bca898a775e6cd9ec8a404f28ba1d6660ddc77))
* use PathBuf for source file ([50d4078](https://github.com/equinor/septic-config-generator/commit/50d4078532544dd6fd4750210e278efb5b656c83))


### ✨ Features

* add .yaml to filename if no extension given ([183d7ba](https://github.com/equinor/septic-config-generator/commit/183d7baf53555460ae9f5b593ed3291f2573d701))
* add command line options with clap ([7c060b4](https://github.com/equinor/septic-config-generator/commit/7c060b47e7ff38b3ede023c5ece17bf04bcc8be2))
* add config option to enforce single newline ([#26](https://github.com/equinor/septic-config-generator/issues/26)) ([63703cf](https://github.com/equinor/septic-config-generator/commit/63703cf9f91514ef414379f95c671c243e5e3ead))
* add gitcommit, gitcommitshort functions ([e42e2f9](https://github.com/equinor/septic-config-generator/commit/e42e2f9e912ef35b85d00770eab0bf52b06eea79))
* add timestamp function now() ([89ecee0](https://github.com/equinor/septic-config-generator/commit/89ecee0f6e4995dff261c79afcd2a47524bac306))
* backup and replace outfile if needed ([#21](https://github.com/equinor/septic-config-generator/issues/21)) ([81e2439](https://github.com/equinor/septic-config-generator/commit/81e243910f7e2f64b53fe21fa05e48d7b82a088c))
* beginning of a potential version in rust ([333ce57](https://github.com/equinor/septic-config-generator/commit/333ce57957714902e710df21f59fa4c3d7340fba))
* better error handling ([067f35e](https://github.com/equinor/septic-config-generator/commit/067f35e8028f938a67f30c134e3c5d9864094bfd))
* get HashMap from source, merge with globals ([393d9bb](https://github.com/equinor/septic-config-generator/commit/393d9bbe013b8a801a50eac212be1e0ff1c5a540))
* improve error reporting from templates ([151f5ca](https://github.com/equinor/septic-config-generator/commit/151f5cac5e98ecaea20ab0b1c87da1357484008b))
* keep row order for source ([576a49a](https://github.com/equinor/septic-config-generator/commit/576a49a96b670834db857db44088256584929fca))
* make now() arg optional, add gitcommitlong ([#31](https://github.com/equinor/septic-config-generator/issues/31)) ([c5b3441](https://github.com/equinor/septic-config-generator/commit/c5b34411f42370d158e73da172ec42079c3f873c))
* make serializable DataType for context ([37ca962](https://github.com/equinor/septic-config-generator/commit/37ca962a7650a980e8a54b147285c8c927025629))
* parse and add globals to environment ([1c33d93](https://github.com/equinor/septic-config-generator/commit/1c33d9319a1e6d12cb56a6f1e5192719d305a231))
* preserve datatype when reading excel sheet ([dab63f7](https://github.com/equinor/septic-config-generator/commit/dab63f71949207df5a89bf0f5471cb1648d879c7))
* read templates as windows-1252 encoded ([7a20ee0](https://github.com/equinor/septic-config-generator/commit/7a20ee0345d911674eb187b9955feca27e00caca))
* render all templates using include/exclude ([ae6ca12](https://github.com/equinor/septic-config-generator/commit/ae6ca12938bed97e2b99f4f58ef7285f8f2d7a5d))
* render to file or to stdout ([bde778f](https://github.com/equinor/septic-config-generator/commit/bde778f64d2aa26634f24107e9df6269dc7a5912))
* render w/custom formatter for missing ctx ([c05f601](https://github.com/equinor/septic-config-generator/commit/c05f60153f763c8adadb58aa85cc7772a1eb6b05))
* use git version for build ([0823697](https://github.com/equinor/septic-config-generator/commit/082369708ceaf2375371b758298d3eba02020521))
* write to encoded file ([1fc43f7](https://github.com/equinor/septic-config-generator/commit/1fc43f7cc12533e022707b1cbac21f59ffd66c13))


### 🐛 Bug Fixes

* better error handling when reading templates ([d3bd1bd](https://github.com/equinor/septic-config-generator/commit/d3bd1bd83349e789dea28f3f93ff2043f440f5a0))
* convert whole numbers to integers ([f3cf755](https://github.com/equinor/septic-config-generator/commit/f3cf75565617e08b4155f60b5580e2bf0313ee1f))
* don't report no diff if file does not exist ([#23](https://github.com/equinor/septic-config-generator/issues/23)) ([834ca38](https://github.com/equinor/septic-config-generator/commit/834ca382b0928953686ad05a694f5642d202b91a))
* handle errors when reading excel as source ([491aa0e](https://github.com/equinor/septic-config-generator/commit/491aa0ef8b16f60df882279d839eebaa8a55989f))
* make --var take key/value pair, use HashMap ([722c77d](https://github.com/equinor/septic-config-generator/commit/722c77ddffbd7b1847348960e4b559a3797074e0))
* make masterpath optional ([d33728a](https://github.com/equinor/septic-config-generator/commit/d33728a6f639c102b6e0d0a365f65e0386839d39))
* print rendered text only when done ([519084e](https://github.com/equinor/septic-config-generator/commit/519084ee81a0e716156ac883759b1e4cab6fbc99))
* remove unused arg --no-verify ([#32](https://github.com/equinor/septic-config-generator/issues/32)) ([c645431](https://github.com/equinor/septic-config-generator/commit/c6454318f18b5b218e682774787cab820a84c8f8))
* update fields, use Yaml 1.2 ([74675ac](https://github.com/equinor/septic-config-generator/commit/74675acfcdb6acbef07545ab27df369a946b6733))
* use cargo version instead of git ([94352e3](https://github.com/equinor/septic-config-generator/commit/94352e3ea0fcba94c8bd1403fac0585d6b75718a))
* use eprintln instead of log for now ([cb5e801](https://github.com/equinor/septic-config-generator/commit/cb5e801ca3d8bdf7bb8d5800cb216b0c299ba5ac))


### 🧹 Chores

* add dependabot.yml ([5c7d4e2](https://github.com/equinor/septic-config-generator/commit/5c7d4e2a5d93f0084649f319b6e7f33e2edecfb5))
* **deps:** bump actions/checkout from 2 to 3 ([#35](https://github.com/equinor/septic-config-generator/issues/35)) ([670109d](https://github.com/equinor/septic-config-generator/commit/670109dcbb0788647a02c40f18d44497b51e84e2))
* **deps:** bump actions/setup-python from 1 to 4 ([#36](https://github.com/equinor/septic-config-generator/issues/36)) ([c7b5968](https://github.com/equinor/septic-config-generator/commit/c7b5968f924e8aa99eca4023c30ab94e895edd1d))
* **deps:** bump clap from 4.1.4 to 4.1.6 ([#38](https://github.com/equinor/septic-config-generator/issues/38)) ([7b4b91e](https://github.com/equinor/septic-config-generator/commit/7b4b91ea24d85177a14a07add5240469e54f9dca))
* **deps:** bump minijinja from 0.30.2 to 0.30.4 ([#37](https://github.com/equinor/septic-config-generator/issues/37)) ([88f23d5](https://github.com/equinor/septic-config-generator/commit/88f23d520a3462ed6c510516c41916ec78d516ec))
* move argument parser into lib.rs ([de96f44](https://github.com/equinor/septic-config-generator/commit/de96f44a9641aa8656413733c8a0cde5fec60fa6))
* move read_source to lib.rs ([03e8472](https://github.com/equinor/septic-config-generator/commit/03e8472588d989fea8715f67b1700c681ed28a83))
* remove unused masterpath and masterkey ([#39](https://github.com/equinor/septic-config-generator/issues/39)) ([fd13c1b](https://github.com/equinor/septic-config-generator/commit/fd13c1b793c26ae55e13f5d2a0df24398467c20a))
* update cargo.lock ([857c83d](https://github.com/equinor/septic-config-generator/commit/857c83dfa093433aaf9a57a23892f04362748ae4))


### 📚 Documentation

* update docs to 2.0 ([#29](https://github.com/equinor/septic-config-generator/issues/29)) ([2e673a1](https://github.com/equinor/septic-config-generator/commit/2e673a16ba29b343ea6f4276c2fd92603d8a3bdc))


### 👷 CI/CD

* add workflows ([#12](https://github.com/equinor/septic-config-generator/issues/12)) ([8e7ac99](https://github.com/equinor/septic-config-generator/commit/8e7ac9940cbaa39a4fd232ec8c4dab96ab1bfe32))
* fix dependabot config ([137c61b](https://github.com/equinor/septic-config-generator/commit/137c61b50799e004d715ae6a30dc93c493292e75))
* fix typo in release-please ([#27](https://github.com/equinor/septic-config-generator/issues/27)) ([15f8962](https://github.com/equinor/septic-config-generator/commit/15f896202980e62fc2d3fb533949206ee63f71fc))
* rename published files from x86_64 to x64 ([#40](https://github.com/equinor/septic-config-generator/issues/40)) ([d1ca8ee](https://github.com/equinor/septic-config-generator/commit/d1ca8ee0d36726a86ad68c7ff88882980d7a7d17))
* replace deprecated workflows ([#15](https://github.com/equinor/septic-config-generator/issues/15)) ([6872a87](https://github.com/equinor/septic-config-generator/commit/6872a8750bf1de6b43d5f8dc43dcc5cfad5c02c1))
* use dot notation in release file names ([1af0083](https://github.com/equinor/septic-config-generator/commit/1af00835c1bc87ba8438895482742272cf78fc6c))
