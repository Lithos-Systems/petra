#!/bin/bash
# Generate MQTT server certificate

set -e

CA_DIR="./ca/root"
MQTT_DIR="./mosquitto/certs"

# Create directories
mkdir -p "$MQTT_DIR"

# Generate server private key
openssl genrsa -out "$MQTT_DIR/server.key" 4096

# Generate certificate signing request
openssl req -new -key "$MQTT_DIR/server.key" -out "$MQTT_DIR/server.csr" \
    -subj "/C=US/ST=Texas/L=Austin/O=Petra Systems/CN=mqtt.petra.systems"

# Create extensions file for server cert
cat > "$MQTT_DIR/server.ext" <<EOF
subjectAltName = DNS:mqtt.petra.systems,DNS:*.mqtt.petra.systems,IP:127.0.0.1
keyUsage = digitalSignature, keyAgreement
extendedKeyUsage = serverAuth
EOF

# Sign with CA
openssl x509 -req -in "$MQTT_DIR/server.csr" \
    -CA "$CA_DIR/ca.crt" -CAkey "$CA_DIR/ca.key" \
    -CAcreateserial -out "$MQTT_DIR/server.crt" \
    -days 365 -sha256 -extfile "$MQTT_DIR/server.ext"

# Set permissions
chmod 644 "$MQTT_DIR/server.crt"
chmod 600 "$MQTT_DIR/server.key"

# Clean up
rm "$MQTT_DIR/server.csr" "$MQTT_DIR/server.ext"

echo "MQTT server certificate generated successfully!"
