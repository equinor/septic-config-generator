# Changelog

## [2.15.0](https://github.com/equinor/septic-config-generator/compare/v2.14.0...v2.15.0) (2026-01-28)


### ✨ Features

* conditional inclusion of single row ([#376](https://github.com/equinor/septic-config-generator/issues/376)) ([09904f6](https://github.com/equinor/septic-config-generator/commit/09904f62294074e52a680545402b2359e04c35b8))


### 🧹 Chores

* **deps:** bump clap from 4.5.40 to 4.5.55 ([#396](https://github.com/equinor/septic-config-generator/issues/396)) ([a8e7ab9](https://github.com/equinor/septic-config-generator/commit/a8e7ab915de41374be209b4dac4fab7bd345bb7f))
* **deps:** bump filetime from 0.2.25 to 0.2.27 ([#395](https://github.com/equinor/septic-config-generator/issues/395)) ([53d7f82](https://github.com/equinor/septic-config-generator/commit/53d7f825bb6440715b1c3e4d878d3896f02f05d6))
* **deps:** bump glob from 0.3.2 to 0.3.3 ([#380](https://github.com/equinor/septic-config-generator/issues/380)) ([35d4eef](https://github.com/equinor/septic-config-generator/commit/35d4eef523bbd2d5b140f8ab6dece9677bb8f20a))
* **deps:** bump minijinja from 2.10.2 to 2.15.1 ([#394](https://github.com/equinor/septic-config-generator/issues/394)) ([a217ed0](https://github.com/equinor/septic-config-generator/commit/a217ed0c1109ac61e0eff00e08b3320b88b7da28))
* **deps:** bump regex from 1.11.1 to 1.12.2 ([#383](https://github.com/equinor/septic-config-generator/issues/383)) ([b334c68](https://github.com/equinor/septic-config-generator/commit/b334c6854ed99c750ab4796ef63fbfef7ffe9d6f))
* **deps:** bump tempfile from 3.20.0 to 3.23.0 ([#378](https://github.com/equinor/septic-config-generator/issues/378)) ([c5ee439](https://github.com/equinor/septic-config-generator/commit/c5ee43992bc0d0ef7f0d7a70a9fd0fe17e8fc4a7))


### 👷 CI/CD

* **deps:** bump actions/checkout from 5 to 6 ([#387](https://github.com/equinor/septic-config-generator/issues/387)) ([a2da8bb](https://github.com/equinor/septic-config-generator/commit/a2da8bb1a3ecb7979b2eae33df29323f8ba5bb70))
* **deps:** bump actions/download-artifact from 5 to 6 ([#379](https://github.com/equinor/septic-config-generator/issues/379)) ([bf54248](https://github.com/equinor/septic-config-generator/commit/bf542485efedde5a6206fd276042ac466e204fc7))
* **deps:** bump actions/download-artifact from 6 to 7 ([#390](https://github.com/equinor/septic-config-generator/issues/390)) ([1a6a516](https://github.com/equinor/septic-config-generator/commit/1a6a5163a3678200c36fd783e87555827a2a1124))
* **deps:** bump actions/upload-artifact from 4 to 6 ([#389](https://github.com/equinor/septic-config-generator/issues/389)) ([1949442](https://github.com/equinor/septic-config-generator/commit/19494420f06a7a18bf3c422382d0f1cc92f13ae0))
* fix workflow permission syntax ([#393](https://github.com/equinor/septic-config-generator/issues/393)) ([7f77a32](https://github.com/equinor/septic-config-generator/commit/7f77a329e2e230d4aaeb4cf1fa3f97192997a3ef))
* update workflow permissions ([#392](https://github.com/equinor/septic-config-generator/issues/392)) ([e9474c2](https://github.com/equinor/septic-config-generator/commit/e9474c25e1cf283102410b56f9f0fa239600ab59))


### 🔨 Refactor

* make clippy happy ([#391](https://github.com/equinor/septic-config-generator/issues/391)) ([6fa99d0](https://github.com/equinor/septic-config-generator/commit/6fa99d0e6cc9fbbdb0b3cf9ffe59902cd0142e27))

## [2.14.0](https://github.com/equinor/septic-config-generator/compare/v2.13.0...v2.14.0) (2025-10-24)


### ✨ Features

* add global filtering of rows from sources ([#333](https://github.com/equinor/septic-config-generator/issues/333)) ([d451a47](https://github.com/equinor/septic-config-generator/commit/d451a473c62a889673458ae5839e4bb03f886b3a))
* add self-documenting json schema for yaml config ([95ddc63](https://github.com/equinor/septic-config-generator/commit/95ddc63a310f3a89eae95bae570e456c6c57a0f1))


### 🧹 Chores

* **deps:** bump anyhow from 1.0.97 to 1.0.100 ([#369](https://github.com/equinor/septic-config-generator/issues/369)) ([a084e7e](https://github.com/equinor/septic-config-generator/commit/a084e7e8d46c32f8e002844578da178ea24672c3))
* **deps:** bump chrono from 0.4.40 to 0.4.42 ([#366](https://github.com/equinor/septic-config-generator/issues/366)) ([9f37804](https://github.com/equinor/septic-config-generator/commit/9f37804e896b1699ab6f74427b96aedd062e0148))
* **deps:** bump flate2 from 1.1.1 to 1.1.4 ([#371](https://github.com/equinor/septic-config-generator/issues/371)) ([5c9d944](https://github.com/equinor/septic-config-generator/commit/5c9d944fa5a78360a6aaae2ddbcc9321048bd4a7))
* **deps:** bump indexmap from 2.9.0 to 2.12.0 ([#374](https://github.com/equinor/septic-config-generator/issues/374)) ([ca42cb1](https://github.com/equinor/septic-config-generator/commit/ca42cb15245e329b61b5531cbca90193418c2ee1))
* **deps:** bump winresource from 0.1.20 to 0.1.23 ([#357](https://github.com/equinor/septic-config-generator/issues/357)) ([5a6bd7b](https://github.com/equinor/septic-config-generator/commit/5a6bd7b1aed51362b11373bb9337afc9f6a2422f))


### 👷 CI/CD

* add explicit permissions to workflows ([#372](https://github.com/equinor/septic-config-generator/issues/372)) ([2c5f6fd](https://github.com/equinor/septic-config-generator/commit/2c5f6fd7284a810186457eb9aa4811df662d1fd2))
* **deps:** bump actions/checkout from 4 to 5 ([#364](https://github.com/equinor/septic-config-generator/issues/364)) ([a2272f8](https://github.com/equinor/septic-config-generator/commit/a2272f8a295234bc2219673ecd20f176fff2f2f3))
* **deps:** bump actions/download-artifact from 4 to 5 ([#362](https://github.com/equinor/septic-config-generator/issues/362)) ([168b545](https://github.com/equinor/septic-config-generator/commit/168b54578bac39c879386c910df782bbd0fef2dd))


### 🔨 Refactor

* make clippy for Rust 1.90.0 happy ([#373](https://github.com/equinor/septic-config-generator/issues/373)) ([2c9e067](https://github.com/equinor/septic-config-generator/commit/2c9e067aa43af4e94eee4f9fd77824573dc3982e))

## [2.13.0](https://github.com/equinor/septic-config-generator/compare/v2.12.0...v2.13.0) (2025-06-11)


### ✨ Features

* add draw.io to csv&png converter ([#343](https://github.com/equinor/septic-config-generator/issues/343)) ([8ced062](https://github.com/equinor/septic-config-generator/commit/8ced062f208c52cf22c45887bc60b5dd864d0717))
* add support for specifying encoding ([#304](https://github.com/equinor/septic-config-generator/issues/304)) ([fa6737a](https://github.com/equinor/septic-config-generator/commit/fa6737a6b34bcb1a64e0ac9c531f49efebfe7447))
* make sources optional ([#332](https://github.com/equinor/septic-config-generator/issues/332)) ([7ba0001](https://github.com/equinor/septic-config-generator/commit/7ba0001d92e7c84cd4d8e82278feca3e6a0c0404))


### 📚 Documentation

* decapitalize Septic, fix example files ([#316](https://github.com/equinor/septic-config-generator/issues/316)) ([702c255](https://github.com/equinor/septic-config-generator/commit/702c2558593293d8a33dfa355f54e15113a823e3))
* remove legacy documentation ([#317](https://github.com/equinor/septic-config-generator/issues/317)) ([1727045](https://github.com/equinor/septic-config-generator/commit/17270457c7f07dc4a588cc6a6bb7e05ebc7f28be))
* update example cnfg files to Septic v3.1.1 ([#312](https://github.com/equinor/septic-config-generator/issues/312)) ([ddf9f5d](https://github.com/equinor/septic-config-generator/commit/ddf9f5dbe198144fc9e022e494d6193801a56ff1))


### 🧹 Chores

* add SECURITY.md ([#299](https://github.com/equinor/septic-config-generator/issues/299)) ([3e05320](https://github.com/equinor/septic-config-generator/commit/3e0532055ff21f56b105d3323ecd5f2028bfd3be))
* **deps:** bump anyhow from 1.0.83 to 1.0.96 ([#308](https://github.com/equinor/septic-config-generator/issues/308)) ([944f047](https://github.com/equinor/septic-config-generator/commit/944f0472c79c85d320009cded96fedce89f54cc4))
* **deps:** bump calamine from 0.24.0 to 0.26.1 ([#313](https://github.com/equinor/septic-config-generator/issues/313)) ([117a853](https://github.com/equinor/septic-config-generator/commit/117a8530a178802c6c49ff9b03f83c03d0ba3e73))
* **deps:** bump clap from 4.5.31 to 4.5.32 ([#328](https://github.com/equinor/septic-config-generator/issues/328)) ([e61fe33](https://github.com/equinor/septic-config-generator/commit/e61fe33d7fa27fa45605f5275f31162cc07e1f9d))
* **deps:** bump clap from 4.5.32 to 4.5.39 ([#349](https://github.com/equinor/septic-config-generator/issues/349)) ([8945d88](https://github.com/equinor/septic-config-generator/commit/8945d885f7c9c6b0a9d7bad21224e60fbeb8c257))
* **deps:** bump clap from 4.5.4 to 4.5.31 ([#303](https://github.com/equinor/septic-config-generator/issues/303)) ([1d31701](https://github.com/equinor/septic-config-generator/commit/1d317016392f53af00079dc79d07105774f80873))
* **deps:** bump colored from 2.1.0 to 3.0.0 ([#309](https://github.com/equinor/septic-config-generator/issues/309)) ([cf2a903](https://github.com/equinor/septic-config-generator/commit/cf2a90336f4862d89a60564e2fc1d217259742e6))
* **deps:** bump curve25519-dalek from 4.1.2 to 4.1.3 ([#287](https://github.com/equinor/septic-config-generator/issues/287)) ([96d9239](https://github.com/equinor/septic-config-generator/commit/96d9239969be7378fff46af3467aebedcb807ab9))
* **deps:** bump diffy from 0.3.0 to 0.4.2 ([#310](https://github.com/equinor/septic-config-generator/issues/310)) ([0aed6e8](https://github.com/equinor/septic-config-generator/commit/0aed6e82ddae9ca05adbbc0b6301bd23c23e5fdc))
* **deps:** bump indexmap from 2.7.1 to 2.9.0 ([#337](https://github.com/equinor/septic-config-generator/issues/337)) ([ee4788f](https://github.com/equinor/septic-config-generator/commit/ee4788f03739f045f1be9a4e2f16c45d44aa0306))
* **deps:** bump minijinja from 2.0.1 to 2.7.0 ([#306](https://github.com/equinor/septic-config-generator/issues/306)) ([a58d7f1](https://github.com/equinor/septic-config-generator/commit/a58d7f1dab20683b958a29f256150ce10643e352))
* **deps:** bump minijinja from 2.8.0 to 2.10.2 ([#345](https://github.com/equinor/septic-config-generator/issues/345)) ([391141f](https://github.com/equinor/septic-config-generator/commit/391141f80a58d97279092c83b509562c768d7fdb))
* **deps:** bump openssl from 0.10.64 to 0.10.71 ([#305](https://github.com/equinor/septic-config-generator/issues/305)) ([d638616](https://github.com/equinor/septic-config-generator/commit/d638616792459f1c21843b08b53113025f335fff))
* **deps:** bump openssl from 0.10.71 to 0.10.72 in the cargo group ([#335](https://github.com/equinor/septic-config-generator/issues/335)) ([79ac32e](https://github.com/equinor/septic-config-generator/commit/79ac32ef55d2d71f40a28be2dbf07c84872683f7))
* **deps:** bump regex from 1.10.4 to 1.11.1 ([#307](https://github.com/equinor/septic-config-generator/issues/307)) ([2cecc76](https://github.com/equinor/septic-config-generator/commit/2cecc7698684c91c79591ece34b478a7c51e86d5))
* **deps:** bump ring from 0.17.11 to 0.17.13 ([#322](https://github.com/equinor/septic-config-generator/issues/322)) ([c44888a](https://github.com/equinor/septic-config-generator/commit/c44888aa7210f1ece5ad32bb55eb18921b6aefd9))
* **deps:** bump self_update from 0.39.0 to 0.42.0 ([#311](https://github.com/equinor/septic-config-generator/issues/311)) ([b06d563](https://github.com/equinor/septic-config-generator/commit/b06d5635989bcee4b3531fdb0a57264434dcf152))
* **deps:** bump tempfile from 3.17.1 to 3.19.0 ([#326](https://github.com/equinor/septic-config-generator/issues/326)) ([70b440a](https://github.com/equinor/septic-config-generator/commit/70b440a2c549badfbf4698a67f1abece1918a315))
* **deps:** bump tempfile from 3.19.0 to 3.20.0 ([#346](https://github.com/equinor/septic-config-generator/issues/346)) ([51212f0](https://github.com/equinor/septic-config-generator/commit/51212f09e6ddfb5343f98bb885f3ca0cc6e7879b))
* **deps:** bump tokio from 1.43.0 to 1.45.0 in the cargo group ([#348](https://github.com/equinor/septic-config-generator/issues/348)) ([bf4a5b6](https://github.com/equinor/septic-config-generator/commit/bf4a5b67eee9545148fb43fd03f1ff787644dfa4))
* **deps:** bump winresource from 0.1.19 to 0.1.20 ([#324](https://github.com/equinor/septic-config-generator/issues/324)) ([34c96f6](https://github.com/equinor/septic-config-generator/commit/34c96f6ca16a748834457d46f1580b1abfd083ae))
* **deps:** bump zip from 2.2.3 to 2.4.1 in the cargo group ([#330](https://github.com/equinor/septic-config-generator/issues/330)) ([ddafead](https://github.com/equinor/septic-config-generator/commit/ddafead0205ee12e6d09b553d6fafd2461bfcf63))
* update Rust edition from 2021 to 2024 ([#318](https://github.com/equinor/septic-config-generator/issues/318)) ([eaf9a0a](https://github.com/equinor/septic-config-generator/commit/eaf9a0ab721549f205807b6bb66e697a82051c01))


### 👷 CI/CD

* add workflow_dispatch for release-please ([#314](https://github.com/equinor/septic-config-generator/issues/314)) ([8e09355](https://github.com/equinor/septic-config-generator/commit/8e093555f757bdab985efdf36e57f03e3b8747b0))
* amend prefix for CI-related dependabot updates ([#329](https://github.com/equinor/septic-config-generator/issues/329)) ([c8eb997](https://github.com/equinor/septic-config-generator/commit/c8eb997a51f358783bc8b36d992079b0e634648e))
* **deps:** bump codecov/codecov-action from 4 to 5 [#320](https://github.com/equinor/septic-config-generator/issues/320) ([152391e](https://github.com/equinor/septic-config-generator/commit/152391e278ae516800ac73dc723165fa7f9969b2))
* **deps:** bump rustsec/audit-check from 1 to 2 [#319](https://github.com/equinor/septic-config-generator/issues/319) ([d20f53e](https://github.com/equinor/septic-config-generator/commit/d20f53e53d65881bcf0ff1325b29713e7a4d1dac))
* fix deprecated release-please source ([#315](https://github.com/equinor/septic-config-generator/issues/315)) ([65c0ba4](https://github.com/equinor/septic-config-generator/commit/65c0ba478533c0c5ef734db2d35523e28fa907e1))


### 🧪 Tests

* fix tempfile deprecation warning ([#351](https://github.com/equinor/septic-config-generator/issues/351)) ([f75fbfa](https://github.com/equinor/septic-config-generator/commit/f75fbfa37bf4e91f1ccb7d3aba72a8c215029f26))

## [2.12.0](https://github.com/equinor/septic-config-generator/compare/v2.11.2...v2.12.0) (2024-05-16)


### ✨ Features

* add conditional includes/excludes for templates ([#277](https://github.com/equinor/septic-config-generator/issues/277)) ([ab097cf](https://github.com/equinor/septic-config-generator/commit/ab097cf4bf1b79a1d7aa6bca2bd39c1b92b0eabd))


### 🧹 Chores

* **deps:** bump anyhow from 1.0.81 to 1.0.83 ([#278](https://github.com/equinor/septic-config-generator/issues/278)) ([15c0ad3](https://github.com/equinor/septic-config-generator/commit/15c0ad3d4e2324bedd3e6084b625bfd9c1b175e0))


### 🔨 Refactor

* bump minijinja from 1.0.20 to 2.0.1 ([#279](https://github.com/equinor/septic-config-generator/issues/279)) ([a5583e5](https://github.com/equinor/septic-config-generator/commit/a5583e55ff0fd4d0da745ca8934df8ba6c068927))

## [2.11.2](https://github.com/equinor/septic-config-generator/compare/v2.11.1...v2.11.2) (2024-04-18)


### 🐛 Bug Fixes

* include filename in yaml error message ([#273](https://github.com/equinor/septic-config-generator/issues/273)) ([5137ddc](https://github.com/equinor/septic-config-generator/commit/5137ddc78d5c60174230d8dc59814074bba00d9e))

## [2.11.1](https://github.com/equinor/septic-config-generator/compare/v2.11.0...v2.11.1) (2024-04-17)


### 🐛 Bug Fixes

* don't panic on undefined source ([#263](https://github.com/equinor/septic-config-generator/issues/263)) ([a9be6ef](https://github.com/equinor/septic-config-generator/commit/a9be6ef89451c2adb57d0cbad533741e268348c5))


### 🧹 Chores

* **deps:** bump chrono from 0.4.37 to 0.4.38 ([#264](https://github.com/equinor/septic-config-generator/issues/264)) ([598a4b6](https://github.com/equinor/septic-config-generator/commit/598a4b6172c7efa9633a0a9f8b2b0f6237603b05))
* **deps:** bump encoding_rs from 0.8.33 to 0.8.34 ([#267](https://github.com/equinor/septic-config-generator/issues/267)) ([cf274c0](https://github.com/equinor/septic-config-generator/commit/cf274c0a71922547ca79b038d12392fe62d64ef0))
* **deps:** bump minijinja from 1.0.16 to 1.0.20 ([#271](https://github.com/equinor/septic-config-generator/issues/271)) ([d3a242c](https://github.com/equinor/septic-config-generator/commit/d3a242c03146d56529e6e459f3943317f15dcc7b))


### 👷 CI/CD

* new attempt to disable annoying codecov ([#269](https://github.com/equinor/septic-config-generator/issues/269)) ([775ef91](https://github.com/equinor/septic-config-generator/commit/775ef91260bddf3c1a24703356df21ba04984bf2))

## [2.11.0](https://github.com/equinor/septic-config-generator/compare/v2.10.0...v2.11.0) (2024-04-09)


### ✨ Features

* add support for multi-file sources (csv) ([6590a6d](https://github.com/equinor/septic-config-generator/commit/6590a6dcbe9c3ba48c54b0554588f6d35238c520))


### 🧹 Chores

* **deps:** bump h2 from 0.3.24 to 0.3.26 ([#253](https://github.com/equinor/septic-config-generator/issues/253)) ([512b1c4](https://github.com/equinor/septic-config-generator/commit/512b1c45e11e31552ec8a78595a191c7cf3f9278))
* **deps:** bump minijinja from 1.0.12 to 1.0.16 ([#255](https://github.com/equinor/septic-config-generator/issues/255)) ([3918b19](https://github.com/equinor/septic-config-generator/commit/3918b192919d85e40a9cf7af0b3773c34dbcdc73))
* **deps:** bump regex from 1.10.3 to 1.10.4 ([#254](https://github.com/equinor/septic-config-generator/issues/254)) ([aebcb84](https://github.com/equinor/septic-config-generator/commit/aebcb84346b7850838be0c6bf3bd9a29f679310d))
* **deps:** bump self_update from 0.38.0 to 0.39.0 ([6590a6d](https://github.com/equinor/septic-config-generator/commit/6590a6dcbe9c3ba48c54b0554588f6d35238c520))
* **deps:** bump tempfile from 3.8.0 to 3.10.1 ([#256](https://github.com/equinor/septic-config-generator/issues/256)) ([da85038](https://github.com/equinor/septic-config-generator/commit/da85038c01a7700b70297f28c2fdf5b2828fb07e))


### 👷 CI/CD

* disable annoying codecov patch fails ([#258](https://github.com/equinor/septic-config-generator/issues/258)) ([fe05c29](https://github.com/equinor/septic-config-generator/commit/fe05c29c8eeed26ac9f29f54dc9bf22005664566))


### 🔨 Refactor

* better error handling and reporting ([14e1e48](https://github.com/equinor/septic-config-generator/commit/14e1e488867e000ffc3c9fbc2059554b35466509))
* clean up lib.rs ([406342f](https://github.com/equinor/septic-config-generator/commit/406342fe6052147fd8a4771335e12351d56168c2))
* clean up overwriting of existing output file ([406342f](https://github.com/equinor/septic-config-generator/commit/406342fe6052147fd8a4771335e12351d56168c2))
* make cmd_*() non-public ([14e1e48](https://github.com/equinor/septic-config-generator/commit/14e1e488867e000ffc3c9fbc2059554b35466509))
* make render_template() a method ([14e1e48](https://github.com/equinor/septic-config-generator/commit/14e1e488867e000ffc3c9fbc2059554b35466509))
* remove unused functions  ([14e1e48](https://github.com/equinor/septic-config-generator/commit/14e1e488867e000ffc3c9fbc2059554b35466509))


### 🧪 Tests

* fix broken test ([e98b77a](https://github.com/equinor/septic-config-generator/commit/e98b77aa08ceb750604b00ad32ebf92d461914a3))
* fix broken tests in Windows ([#260](https://github.com/equinor/septic-config-generator/issues/260)) ([e6443a4](https://github.com/equinor/septic-config-generator/commit/e6443a4ce99aaf103cc4efe7078acb6f2b025b54))

## [2.10.0](https://github.com/equinor/septic-config-generator/compare/v2.9.1...v2.10.0) (2024-04-02)


### ✨ Features

* pass global sources as vecs, not hashmaps ([#249](https://github.com/equinor/septic-config-generator/issues/249)) ([a97d2c8](https://github.com/equinor/septic-config-generator/commit/a97d2c87204e787696930544e67db6f0f0a0bc5f))


### 🐛 Bug Fixes

* polluted ctx for non-iteration templates ([#248](https://github.com/equinor/septic-config-generator/issues/248)) ([56d8a35](https://github.com/equinor/septic-config-generator/commit/56d8a354e8cd1ea215e1c903dd0755f13111f005))


### 🧹 Chores

* **deps:** bump calamine from 0.22.1 to 0.24.0 ([#230](https://github.com/equinor/septic-config-generator/issues/230)) ([b83b293](https://github.com/equinor/septic-config-generator/commit/b83b29370f07a80cabcf63abc1e7f0ea1f2914f0))
* **deps:** bump chrono from 0.4.34 to 0.4.37 ([#246](https://github.com/equinor/septic-config-generator/issues/246)) ([6caa540](https://github.com/equinor/septic-config-generator/commit/6caa540c59f07e2fc50c8c1984356b9c3a84e2fa))
* **deps:** bump clap from 4.4.18 to 4.5.4 ([#245](https://github.com/equinor/septic-config-generator/issues/245)) ([e5b933f](https://github.com/equinor/septic-config-generator/commit/e5b933fca4f7b5406ff878abfbf67aa5e45d3eea))
* **deps:** bump indexmap from 2.2.3 to 2.2.6 ([#244](https://github.com/equinor/septic-config-generator/issues/244)) ([7589f27](https://github.com/equinor/septic-config-generator/commit/7589f2706ea6fde613322030649c1c6b779996ca))
* **deps:** bump mio from 0.8.8 to 0.8.11 ([#234](https://github.com/equinor/septic-config-generator/issues/234)) ([a0d9f4e](https://github.com/equinor/septic-config-generator/commit/a0d9f4e579f564b55b303add63d5f86cd404e0f0))
* **deps:** bump serde_yaml from 0.9.32 to 0.9.34+deprecated ([#243](https://github.com/equinor/septic-config-generator/issues/243)) ([78d21b3](https://github.com/equinor/septic-config-generator/commit/78d21b3f0a19c5ee0cc99eb208332e371ae13963))
* **deps:** bump softprops/action-gh-release from 1 to 2 ([#237](https://github.com/equinor/septic-config-generator/issues/237)) ([07a96cd](https://github.com/equinor/septic-config-generator/commit/07a96cd8351f4373db7ca7a77b9cf4197ef33b68))


### 👷 CI/CD

* fix "perf" commit type for release-please ([#247](https://github.com/equinor/septic-config-generator/issues/247)) ([6052e34](https://github.com/equinor/septic-config-generator/commit/6052e3416511de28e23cd9a19e42ce25eeb8efde))


### 🏎️ Performance

* add sources once as globals ([#236](https://github.com/equinor/septic-config-generator/issues/236)) ([210a4d1](https://github.com/equinor/septic-config-generator/commit/210a4d1f37435dfd963eb87c68a04c63deacbba5))

## [2.9.1](https://github.com/equinor/septic-config-generator/compare/v2.9.0...v2.9.1) (2024-02-27)


### 🐛 Bug Fixes

* unpack output non-array value for single argument ([bbe8425](https://github.com/equinor/septic-config-generator/commit/bbe8425eda10d7511241cab0b9c1f779372354ae))


### 📚 Documentation

* improve docs for filters ([#218](https://github.com/equinor/septic-config-generator/issues/218)) ([9d1d77e](https://github.com/equinor/septic-config-generator/commit/9d1d77ecc3ddb3aa44a5956f67e09fe97675aa91))


### 🧹 Chores

* **deps:** bump clap from 4.4.6 to 4.4.18 ([#222](https://github.com/equinor/septic-config-generator/issues/222)) ([07f9208](https://github.com/equinor/septic-config-generator/commit/07f9208f8ec1914f343182cb7a33caec9646165c))
* **deps:** bump codecov/codecov-action from 3 to 4 ([#226](https://github.com/equinor/septic-config-generator/issues/226)) ([a5f4e18](https://github.com/equinor/septic-config-generator/commit/a5f4e1838809479de428c56469b15b767c596e02))
* **deps:** bump colored from 2.0.4 to 2.1.0 ([#223](https://github.com/equinor/septic-config-generator/issues/223)) ([d3f1444](https://github.com/equinor/septic-config-generator/commit/d3f14441383e4228343df677339f320772f88d20))
* **deps:** bump regex from 1.9.4 to 1.10.3 ([#221](https://github.com/equinor/septic-config-generator/issues/221)) ([83b2c24](https://github.com/equinor/septic-config-generator/commit/83b2c24cb33afde4c343e7b46d4f60887de446d0))
* **deps:** bump serde_yaml from 0.9.25 to 0.9.32 ([#224](https://github.com/equinor/septic-config-generator/issues/224)) ([a8265eb](https://github.com/equinor/septic-config-generator/commit/a8265ebdfd0498be77931909d7e654642604cc82))


### 👷 CI/CD

* only test when updating relevant files ([#220](https://github.com/equinor/septic-config-generator/issues/220)) ([75aef70](https://github.com/equinor/septic-config-generator/commit/75aef70ec2e3e3feae2d4af4604f02a13f94df0a))

## [2.9.0](https://github.com/equinor/septic-config-generator/compare/v2.8.0...v2.9.0) (2024-02-25)


### ✨ Features

* add filter to extract values from hashmap ([#214](https://github.com/equinor/septic-config-generator/issues/214)) ([304b3aa](https://github.com/equinor/septic-config-generator/commit/304b3aa348638620d83a6eadd09708a58c25f694))
* add filter to unpack values from maps ([#216](https://github.com/equinor/septic-config-generator/issues/216)) ([e1bbaeb](https://github.com/equinor/septic-config-generator/commit/e1bbaeb224780e71c81d5a0e5feb8a2f6eb84507))


### 🧹 Chores

* **deps:** bump actions/download-artifact from 3 to 4 ([#196](https://github.com/equinor/septic-config-generator/issues/196)) ([83de9d8](https://github.com/equinor/septic-config-generator/commit/83de9d8f1dbf738c593c64498799fe54107d5788))
* **deps:** bump actions/upload-artifact from 3 to 4 ([#197](https://github.com/equinor/septic-config-generator/issues/197)) ([fa68c78](https://github.com/equinor/septic-config-generator/commit/fa68c78b3d36b63f4c57df1428bb28e7168fd72e))
* **deps:** bump google-github-actions/release-please-action from 3 to 4 ([#195](https://github.com/equinor/septic-config-generator/issues/195)) ([b599c5a](https://github.com/equinor/septic-config-generator/commit/b599c5a0a6950d6a44b5076155272ba9136423ac))
* update release-please-action from 3 to 4 ([#217](https://github.com/equinor/septic-config-generator/issues/217)) ([954089e](https://github.com/equinor/septic-config-generator/commit/954089e2df9bc8172f155a1d8b871597e53ca30c))

## [2.8.0](https://github.com/equinor/septic-config-generator/compare/v2.7.0...v2.8.0) (2024-02-20)


### 🧹 Chores

* **deps:** bump calamine from 0.22.0 to 0.22.1 ([#185](https://github.com/equinor/septic-config-generator/issues/185)) ([7c1d095](https://github.com/equinor/septic-config-generator/commit/7c1d09514fd61b06cbbd4450672196b40a971043))
* **deps:** bump chrono from 0.4.26 to 0.4.34 ([#211](https://github.com/equinor/septic-config-generator/issues/211)) ([2e03316](https://github.com/equinor/septic-config-generator/commit/2e03316f3e14edfc506fea2ad3b9dafbed84b811))
* **deps:** bump csv from 1.2.2 to 1.3.0 ([#186](https://github.com/equinor/septic-config-generator/issues/186)) ([08a035b](https://github.com/equinor/septic-config-generator/commit/08a035befbfb96bb4f1a5928b726190bf51f81de))
* **deps:** bump h2 from 0.3.21 to 0.3.24 ([#210](https://github.com/equinor/septic-config-generator/issues/210)) ([16b8371](https://github.com/equinor/septic-config-generator/commit/16b83717a3aac48ca0d948d3fa808f22ada76317))
* **deps:** bump minijinja from 1.0.8 to 1.0.12 ([#209](https://github.com/equinor/septic-config-generator/issues/209)) ([be16ea6](https://github.com/equinor/septic-config-generator/commit/be16ea62ec9e081afb191d392db68b304c0c1b3c))
* **deps:** bump openssl from 0.10.57 to 0.10.64 ([#207](https://github.com/equinor/septic-config-generator/issues/207)) ([5985a07](https://github.com/equinor/septic-config-generator/commit/5985a0793bbd25a92a8917c1a68d12df345a3f9f))
* **deps:** bump regex from 1.9.3 to 1.9.4 ([#188](https://github.com/equinor/septic-config-generator/issues/188)) ([88112d0](https://github.com/equinor/septic-config-generator/commit/88112d079ec46dd0297b6a74164f0f75a2b14e89))
* **deps:** bump self_cell from 1.0.1 to 1.0.3 ([#208](https://github.com/equinor/septic-config-generator/issues/208)) ([260d878](https://github.com/equinor/septic-config-generator/commit/260d878bee157231343962e184a891fc91cdd53e))
* **deps:** bump unsafe-libyaml from 0.2.9 to 0.2.10 ([#198](https://github.com/equinor/septic-config-generator/issues/198)) ([9e21a5d](https://github.com/equinor/septic-config-generator/commit/9e21a5d3e01cf548c542c984b663fe90ba3c0eb7))


### ✨ Features

* make all sources available to all templates ([#204](https://github.com/equinor/septic-config-generator/issues/204)) ([6a3256a](https://github.com/equinor/septic-config-generator/commit/6a3256ad9e49a89c61aa9c73e874a6ea60477bbe))


### 🧪 Tests

* add test for csv fields beginning with "0" ([#205](https://github.com/equinor/septic-config-generator/issues/205)) ([f7e3c85](https://github.com/equinor/septic-config-generator/commit/f7e3c85ed6016bb8a4e18ba408214e5a82dfd715))

## [2.7.0](https://github.com/equinor/septic-config-generator/compare/v2.6.1...v2.7.0) (2023-10-11)


### 🧹 Chores

* **deps:** bump clap from 4.3.21 to 4.4.6 ([#170](https://github.com/equinor/septic-config-generator/issues/170)) ([4c5d501](https://github.com/equinor/septic-config-generator/commit/4c5d50197184d2bc83c2c9ae2cf97da7224886e5))
* **deps:** bump encoding_rs from 0.8.32 to 0.8.33 ([#156](https://github.com/equinor/septic-config-generator/issues/156)) ([9c5e20e](https://github.com/equinor/septic-config-generator/commit/9c5e20e845a986a449539e88b91f03672c5e341f))
* **deps:** bump minijinja from 1.0.5 to 1.0.8 ([#166](https://github.com/equinor/septic-config-generator/issues/166)) ([958dd0a](https://github.com/equinor/septic-config-generator/commit/958dd0a40afb092ec2a4b53bcca06be1ef866b1f))
* **deps:** bump tempfile from 3.7.1 to 3.8.0 ([#155](https://github.com/equinor/septic-config-generator/issues/155)) ([d6d8ce9](https://github.com/equinor/septic-config-generator/commit/d6d8ce90ba3b726356213525e5e94bbc4e20ea3e))
* **deps:** bump winresource from 0.1.16 to 0.1.17 ([#157](https://github.com/equinor/septic-config-generator/issues/157)) ([14400ba](https://github.com/equinor/septic-config-generator/commit/14400ba527aaf6a9c05f0815dbf00f8fa58c47a8))


### 📚 Documentation

* update for v2.6 ([#181](https://github.com/equinor/septic-config-generator/issues/181)) ([c77257e](https://github.com/equinor/septic-config-generator/commit/c77257e845d25a5a230972cd38a6821b4bc737ed))


### ✨ Features

* add global auto-incrementing counters ([#183](https://github.com/equinor/septic-config-generator/issues/183)) ([40ef2e1](https://github.com/equinor/septic-config-generator/commit/40ef2e150832beafad8fd43b75b5469653c6093f))

## [2.6.1](https://github.com/equinor/septic-config-generator/compare/v2.6.0...v2.6.1) (2023-10-10)


### 👷 CI/CD

* don't compile for musl target ([ba111e6](https://github.com/equinor/septic-config-generator/commit/ba111e67294e03236b5ce5997f90f2cc65d5d633))

## [2.6.0](https://github.com/equinor/septic-config-generator/compare/v2.5.2...v2.6.0) (2023-10-10)


### 👷 CI/CD

* make binary executable ([#177](https://github.com/equinor/septic-config-generator/issues/177)) ([07e76f5](https://github.com/equinor/septic-config-generator/commit/07e76f522ba16ca85b58709d9cf1c22011d90a47))


### ✨ Features

* add command to self update ([#175](https://github.com/equinor/septic-config-generator/issues/175)) ([1a4f9ec](https://github.com/equinor/septic-config-generator/commit/1a4f9ecb6fcaa13fad19ac0cbb3f8727cc874ba2))

## [2.5.2](https://github.com/equinor/septic-config-generator/compare/v2.5.1...v2.5.2) (2023-10-09)


### 🧹 Chores

* add badges ([#150](https://github.com/equinor/septic-config-generator/issues/150)) ([b09a621](https://github.com/equinor/septic-config-generator/commit/b09a621ab8aa31288299ed64c5535663619ae56b))
* **deps:** bump actions/checkout from 3 to 4 ([#162](https://github.com/equinor/septic-config-generator/issues/162)) ([865a8c2](https://github.com/equinor/septic-config-generator/commit/865a8c21ac59a1e5e2583f729d96203c5a3e4e94))
* **deps:** bump calamine from 0.21.2 to 0.22.0 ([#164](https://github.com/equinor/septic-config-generator/issues/164)) ([9b7367a](https://github.com/equinor/septic-config-generator/commit/9b7367aa695fce11f071aa15c6e191986e26459c))
* remove legacy python code ([#153](https://github.com/equinor/septic-config-generator/issues/153)) ([18cfbf5](https://github.com/equinor/septic-config-generator/commit/18cfbf53de5ab9e4e35811333f2fe447a23f6ec3))


### 👷 CI/CD

* add build targets for aarch64 macos + linux musl ([#176](https://github.com/equinor/septic-config-generator/issues/176)) ([0ce9fcf](https://github.com/equinor/septic-config-generator/commit/0ce9fcf52071108006030ebbfbd1f8cba710ea7a))


### 🔨 Refactor

* move commands into separate files ([#173](https://github.com/equinor/septic-config-generator/issues/173)) ([56d69c3](https://github.com/equinor/septic-config-generator/commit/56d69c3c6c2a158bcb372ede198c965247d57f4c))

## [2.5.1](https://github.com/equinor/septic-config-generator/compare/v2.5.0...v2.5.1) (2023-08-10)


### 🧹 Chores

* **deps:** bump calamine from 1.19.1 to 1.21.2 ([#147](https://github.com/equinor/septic-config-generator/issues/147)) ([22d9593](https://github.com/equinor/septic-config-generator/commit/22d95934f71bae790fd76a54264781758dc7344e))
* **deps:** bump clap from 4.3.3 to 4.3.19 ([#139](https://github.com/equinor/septic-config-generator/issues/139)) ([26eec98](https://github.com/equinor/septic-config-generator/commit/26eec98252852676aebf3d8cc16b2c8727bb0a9b))
* **deps:** bump colored from 2.0.0 to 2.0.4 ([#134](https://github.com/equinor/septic-config-generator/issues/134)) ([7b11eca](https://github.com/equinor/septic-config-generator/commit/7b11eca75b0f9208741129a223a2f53f20c9f518))
* **deps:** bump minijinja from 0.30.7 to 1.0.5 ([#148](https://github.com/equinor/septic-config-generator/issues/148)) ([859787b](https://github.com/equinor/septic-config-generator/commit/859787b8620b193098ac9552220796702c65f374))
* **deps:** bump winresource from 0.1.15 to 0.1.16 ([#138](https://github.com/equinor/septic-config-generator/issues/138)) ([4713238](https://github.com/equinor/septic-config-generator/commit/471323851a18d6d91c66511c7b07632224891bc5))


### 👷 CI/CD

* fix codecov threshold ([#146](https://github.com/equinor/septic-config-generator/issues/146)) ([ee3a654](https://github.com/equinor/septic-config-generator/commit/ee3a65416428266ff47e523babbe423a311528a0))
* test + cov for push and pr, test before build ([#143](https://github.com/equinor/septic-config-generator/issues/143)) ([7b1e8e8](https://github.com/equinor/septic-config-generator/commit/7b1e8e891647fb4e88c2ed95f7cc86e7996b7683))


### 🐛 Bug Fixes

* comma as decimal separator in csv ([#145](https://github.com/equinor/septic-config-generator/issues/145)) ([37be222](https://github.com/equinor/septic-config-generator/commit/37be222060b8fe319c80225d9b1ebe7e5c498b49))
* **csv:** parse 01, 02 etc as string ([#149](https://github.com/equinor/septic-config-generator/issues/149)) ([d4691e4](https://github.com/equinor/septic-config-generator/commit/d4691e49c0742f7ec55ddf4b4f34567e11345732))


### 🔨 Refactor

* make clippy pass all nursery lints ([#144](https://github.com/equinor/septic-config-generator/issues/144)) ([c25e09c](https://github.com/equinor/septic-config-generator/commit/c25e09c3057ddb3824752d1b83a4e3b7b01dff44))

## [2.5.0](https://github.com/equinor/septic-config-generator/compare/v2.4.0...v2.5.0) (2023-06-12)


### ✨ Features

* use csv file as data source ([2f5f9b0](https://github.com/equinor/septic-config-generator/commit/2f5f9b06132783c5795a80ba273a4a2c32c4f48d))


### 🧪 Tests

* reorganize and rename tests ([2f5f9b0](https://github.com/equinor/septic-config-generator/commit/2f5f9b06132783c5795a80ba273a4a2c32c4f48d))


### 🐛 Bug Fixes

* do not accept unknown yaml fields ([2f5f9b0](https://github.com/equinor/septic-config-generator/commit/2f5f9b06132783c5795a80ba273a4a2c32c4f48d))


### 👷 CI/CD

* create codecov.yml ([#123](https://github.com/equinor/septic-config-generator/issues/123)) ([cccf4f8](https://github.com/equinor/septic-config-generator/commit/cccf4f84cc71a950b8a5565be29434d5a972c4b0))
* grant permissions for Dependabot and security audit ([#128](https://github.com/equinor/septic-config-generator/issues/128)) ([7fdbfc7](https://github.com/equinor/septic-config-generator/commit/7fdbfc79cb395ecda47b43229d653ac154fa5505))


### 🧹 Chores

* **deps:** bump clap from 4.3.1 to 4.3.3 ([#125](https://github.com/equinor/septic-config-generator/issues/125)) ([e2d1433](https://github.com/equinor/septic-config-generator/commit/e2d14336cf136c061b0b4ae83079975cb4e6f5b6))
* **deps:** bump regex from 1.8.3 to 1.8.4 ([#126](https://github.com/equinor/septic-config-generator/issues/126)) ([6414af9](https://github.com/equinor/septic-config-generator/commit/6414af9bab233ae6c2659bd464c10594a250c07e))
* **deps:** bump tempfile from 3.5.0 to 3.6.0 ([#127](https://github.com/equinor/septic-config-generator/issues/127)) ([63eaf36](https://github.com/equinor/septic-config-generator/commit/63eaf36566c67ae3e0e4c188497a52a0dc190b08))


### 📚 Documentation

* update documentation for 2.5 ([#120](https://github.com/equinor/septic-config-generator/issues/120)) ([ca673ab](https://github.com/equinor/septic-config-generator/commit/ca673ab688f0deb4c1bfe60147c3b3b1cb37baf7))

## [2.4.0](https://github.com/equinor/septic-config-generator/compare/v2.3.1...v2.4.0) (2023-06-09)


### 🧪 Tests

* better tests for bitmask() ([#107](https://github.com/equinor/septic-config-generator/issues/107)) ([da13074](https://github.com/equinor/septic-config-generator/commit/da13074a8d78d0bb7201ceffe1aa59b0b4cbc873))


### 👷 CI/CD

* caching in GitHub workflows not working properly ([4b6e695](https://github.com/equinor/septic-config-generator/commit/4b6e695a155d524b60d83f93939f26d607bdbc60))


### ✨ Features

* add command 'checklogs' ([#61](https://github.com/equinor/septic-config-generator/issues/61)) ([d38f5ac](https://github.com/equinor/septic-config-generator/commit/d38f5ac41240bfc268dbad05a29f27a42baf7569))
* embed Windows metadata ([#90](https://github.com/equinor/septic-config-generator/issues/90)) ([8b578f9](https://github.com/equinor/septic-config-generator/commit/8b578f9c0c3a65edb7136c9c28e157cf4385bc87))


### 🐛 Bug Fixes

* checklogs exit w/code 1 on match, 2 on error ([#117](https://github.com/equinor/septic-config-generator/issues/117)) ([515f010](https://github.com/equinor/septic-config-generator/commit/515f010d9900f7ffa1fd086420181fb18ba66b9a))
* potential segfault in the time crate [RUSTSEC-2020-0071] ([#109](https://github.com/equinor/septic-config-generator/issues/109)) ([752c85d](https://github.com/equinor/septic-config-generator/commit/752c85dd96b2879a7024f046897074b09a55a5aa))


### 📚 Documentation

* update documentation for 2.4.0 ([#114](https://github.com/equinor/septic-config-generator/issues/114)) ([ff847ea](https://github.com/equinor/septic-config-generator/commit/ff847ea2eabb31bab53d46e68a59e6e79a77f6a1))


### 🧹 Chores

* **deps:** bump clap from 4.3.0 to 4.3.1 ([#115](https://github.com/equinor/septic-config-generator/issues/115)) ([dead684](https://github.com/equinor/septic-config-generator/commit/dead68482353372977533f68ef7654b1ed007cef))

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
