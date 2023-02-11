# Changelog

## [1.1.0](https://github.com/einarsi/septic-config-generator/compare/v1.0.0...v1.1.0) (2023-02-11)


### 📦 Build system

* rename binary to scg ([d083f24](https://github.com/einarsi/septic-config-generator/commit/d083f2416d8ce6ae57a2c68927c9b806e2bb7624))


### 💎 Style

* disable colors ([1e95b0f](https://github.com/einarsi/septic-config-generator/commit/1e95b0f525a12597d475875ee7c65c7f8b689a5f))


### 🧪 Tests

* add some test templates ([848f00a](https://github.com/einarsi/septic-config-generator/commit/848f00ace4b03722f76d2e7bcf9e682fa10b7318))
* add test excel file ([e752533](https://github.com/einarsi/septic-config-generator/commit/e7525339731420251a2f108db76e1ee23f4cf898))


### 🐛 Bug Fixes

* better error handling when reading templates ([d3bd1bd](https://github.com/einarsi/septic-config-generator/commit/d3bd1bd83349e789dea28f3f93ff2043f440f5a0))
* convert whole numbers to integers ([f3cf755](https://github.com/einarsi/septic-config-generator/commit/f3cf75565617e08b4155f60b5580e2bf0313ee1f))
* handle errors when reading excel as source ([491aa0e](https://github.com/einarsi/septic-config-generator/commit/491aa0ef8b16f60df882279d839eebaa8a55989f))
* make --var take key/value pair, use HashMap ([722c77d](https://github.com/einarsi/septic-config-generator/commit/722c77ddffbd7b1847348960e4b559a3797074e0))
* make masterpath optional ([d33728a](https://github.com/einarsi/septic-config-generator/commit/d33728a6f639c102b6e0d0a365f65e0386839d39))
* print rendered text only when done ([519084e](https://github.com/einarsi/septic-config-generator/commit/519084ee81a0e716156ac883759b1e4cab6fbc99))
* update fields, use Yaml 1.2 ([74675ac](https://github.com/einarsi/septic-config-generator/commit/74675acfcdb6acbef07545ab27df369a946b6733))
* use eprintln instead of log for now ([cb5e801](https://github.com/einarsi/septic-config-generator/commit/cb5e801ca3d8bdf7bb8d5800cb216b0c299ba5ac))


### 🧹 Chores

* move argument parser into lib.rs ([de96f44](https://github.com/einarsi/septic-config-generator/commit/de96f44a9641aa8656413733c8a0cde5fec60fa6))
* move read_source to lib.rs ([03e8472](https://github.com/einarsi/septic-config-generator/commit/03e8472588d989fea8715f67b1700c681ed28a83))
* update cargo.lock ([857c83d](https://github.com/einarsi/septic-config-generator/commit/857c83dfa093433aaf9a57a23892f04362748ae4))


### 🔨 Refactor

* add add_globals() ([e96a9ea](https://github.com/einarsi/septic-config-generator/commit/e96a9eae24b44b0932967cf48e10f251a98e574d))
* add function to bubble minijinja errors ([2068737](https://github.com/einarsi/septic-config-generator/commit/2068737e9966c86dfad06032423da94ab3e0e260))
* create CtxDataType ([f2df1c1](https://github.com/einarsi/septic-config-generator/commit/f2df1c19db9774becd1a2b5a49ec1f1bfd6b8e1f))
* extract closure into load_template() ([2c10c1c](https://github.com/einarsi/septic-config-generator/commit/2c10c1c5a2414a0802a41e433f7d09ba7b6540cf))
* follow clippy advice ([a0f7875](https://github.com/einarsi/septic-config-generator/commit/a0f78752263981be5cc29fbe6a6be0e3d2be5eb0))
* impl config read, simpler error handling ([0b62cb8](https://github.com/einarsi/septic-config-generator/commit/0b62cb8082523aac3c1bca20d2025f04dada2f11))
* let renderer do the rendering ([9746c79](https://github.com/einarsi/septic-config-generator/commit/9746c79a868a7809ab60c775cd1bc82e8909b2bd))
* make gitcommit a global variable ([e82c0d4](https://github.com/einarsi/septic-config-generator/commit/e82c0d4fd54ef10918dd80b49a3880a039571d23))
* move jinja-stuff into separate module ([daed376](https://github.com/einarsi/septic-config-generator/commit/daed3767d8aab791fe50a9963642f58d6f4fd485))
* move make command into own function ([f01ad94](https://github.com/einarsi/septic-config-generator/commit/f01ad940c4d87dd3a0a34883bc123ca0fbf6fec2))
* move yaml parsing to lib.rs ([fec9164](https://github.com/einarsi/septic-config-generator/commit/fec9164b70964fbea86b54dc9271efd6d214d73f))
* rename libs, separate 2 modules ([3308f31](https://github.com/einarsi/septic-config-generator/commit/3308f31ddc2ed385f6a2a59be31144394da4ecac))
* split config read into own function ([5219c97](https://github.com/einarsi/septic-config-generator/commit/5219c975c6d0886c05d1c2d69cf5fefca724d6ee))
* split excel read into own function ([ffc5905](https://github.com/einarsi/septic-config-generator/commit/ffc59050e906b3e1c4b38fa421dce778a0ddf59d))
* split lib.rs into multiple files ([89bca89](https://github.com/einarsi/septic-config-generator/commit/89bca898a775e6cd9ec8a404f28ba1d6660ddc77))
* use PathBuf for source file ([50d4078](https://github.com/einarsi/septic-config-generator/commit/50d4078532544dd6fd4750210e278efb5b656c83))


### ✨ Features

* add .yaml to filename if no extension given ([183d7ba](https://github.com/einarsi/septic-config-generator/commit/183d7baf53555460ae9f5b593ed3291f2573d701))
* add command line options with clap ([7c060b4](https://github.com/einarsi/septic-config-generator/commit/7c060b47e7ff38b3ede023c5ece17bf04bcc8be2))
* add gitcommit, gitcommitshort functions ([e42e2f9](https://github.com/einarsi/septic-config-generator/commit/e42e2f9e912ef35b85d00770eab0bf52b06eea79))
* add timestamp function now() ([89ecee0](https://github.com/einarsi/septic-config-generator/commit/89ecee0f6e4995dff261c79afcd2a47524bac306))
* beginning of a potential version in rust ([333ce57](https://github.com/einarsi/septic-config-generator/commit/333ce57957714902e710df21f59fa4c3d7340fba))
* better error handling ([067f35e](https://github.com/einarsi/septic-config-generator/commit/067f35e8028f938a67f30c134e3c5d9864094bfd))
* get HashMap from source, merge with globals ([393d9bb](https://github.com/einarsi/septic-config-generator/commit/393d9bbe013b8a801a50eac212be1e0ff1c5a540))
* improve error reporting from templates ([151f5ca](https://github.com/einarsi/septic-config-generator/commit/151f5cac5e98ecaea20ab0b1c87da1357484008b))
* keep row order for source ([576a49a](https://github.com/einarsi/septic-config-generator/commit/576a49a96b670834db857db44088256584929fca))
* make serializable DataType for context ([37ca962](https://github.com/einarsi/septic-config-generator/commit/37ca962a7650a980e8a54b147285c8c927025629))
* parse and add globals to environment ([1c33d93](https://github.com/einarsi/septic-config-generator/commit/1c33d9319a1e6d12cb56a6f1e5192719d305a231))
* preserve datatype when reading excel sheet ([dab63f7](https://github.com/einarsi/septic-config-generator/commit/dab63f71949207df5a89bf0f5471cb1648d879c7))
* read templates as windows-1252 encoded ([7a20ee0](https://github.com/einarsi/septic-config-generator/commit/7a20ee0345d911674eb187b9955feca27e00caca))
* render all templates using include/exclude ([ae6ca12](https://github.com/einarsi/septic-config-generator/commit/ae6ca12938bed97e2b99f4f58ef7285f8f2d7a5d))
* render to file or to stdout ([bde778f](https://github.com/einarsi/septic-config-generator/commit/bde778f64d2aa26634f24107e9df6269dc7a5912))
* render w/custom formatter for missing ctx ([c05f601](https://github.com/einarsi/septic-config-generator/commit/c05f60153f763c8adadb58aa85cc7772a1eb6b05))
* use git version for build ([0823697](https://github.com/einarsi/septic-config-generator/commit/082369708ceaf2375371b758298d3eba02020521))
* write to encoded file ([1fc43f7](https://github.com/einarsi/septic-config-generator/commit/1fc43f7cc12533e022707b1cbac21f59ffd66c13))


### 👷 CI/CD

* create dependabot.yml ([8b5bf30](https://github.com/einarsi/septic-config-generator/commit/8b5bf302e45e4c3a561e1bd1a0a312d8d499144e))
