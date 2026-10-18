mkdir -p certs && mkcert -install && mkcert -ecdsa -cert-file certs/cert.pem -key-file certs/key.pem localhost 127.0.0.1 ::1 0.0.0.0
