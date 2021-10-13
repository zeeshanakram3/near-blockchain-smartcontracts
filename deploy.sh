#!/bin/bash

TARGET_PATH="src/contracts/target/wasm32-unknown-unknown/release"
near deploy --wasmFile "$TARGET_PATH"/ft.wasm --accountId ft.momentize.testnet
near deploy --wasmFile "$TARGET_PATH"/nft.wasm --accountId nft.momentize.testnet
near deploy --wasmFile "$TARGET_PATH"/marketplace.wasm --accountId marketplace.momentize.testnet
near deploy --wasmFile "$TARGET_PATH"/st.wasm --accountId st.momentize.testnet
near deploy --wasmFile "$TARGET_PATH"/oracle.wasm --accountId oracle.momentize.testnet
near deploy --wasmFile "$TARGET_PATH"/usecases.wasm --accountId usecases.momentize.testnet