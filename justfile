gen:
    cargo run --bin gen

run:
    cargo run --bin odata_playground

watch:
    cargo watch -x "run --bin gen Alt_Address_Card"
    # cargo watch -x "run --bin odata_playground"
