# Changelog

All notable changes to this project will be documented in this file.

This project adheres to [Semantic Versioning](http://semver.org/spec/v2.0.0.html).


## 0.2.2 - 2020-11-21

- [fix] Fixed bug in the stateful debouncer initialization (#7)

## 0.2.1 - 2020-11-18

- [fix] Docs: Fix typo in RTIC example

## 0.2.0 - 2020-11-03

By default, the debouncer will report any change from "bouncing" to "stable
high/low" as an edge. If instead you want to detect only changes from a stable
state to the opposite stable state, use the new stateful debouncer instead.

Additionally, the debouncer construction function now allows specifying the
initial state.

- [add] Implement stateful debouncing (#3)
- [add] Allow specifying initial state (#5)

## 0.1.3 - 2020-08-20

- [fix] Docs-only update

## 0.1.2 - 2020-04-28

- [fix] Fix documentation examples

## 0.1.1 - 2020-04-28

- [fix] Fix metadata in Cargo.toml

## 0.1.0 - 2020-04-28

This is the initial release to crates.io. All changes will be documented in
this CHANGELOG.
