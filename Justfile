run *args:
    cargo run -- {{ args }}

tunnel *args:
    ssh -L 8081:localhost:8081 jarvis
