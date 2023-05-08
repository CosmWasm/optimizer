#!/bin/bash

# Read Cargo.toml into a variable
TOML=$(cat Cargo.toml)

# Extract feature keys from Cargo.toml
FEATURE_KEYS=$(sed -n '/^\[features\]/,/^$/p' Cargo.toml | grep -oE '^\w+' | grep -Ev '^\s*(#|$)')

# Build code for each feature
for FEATURE in $FEATURE_KEYS; do
    echo "Building with feature: $FEATURE"
    cargo build --features "$FEATURE"
done
