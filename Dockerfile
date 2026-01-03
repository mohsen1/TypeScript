FROM node:lts

# Install Rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

# Install wasm-pack
RUN curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

WORKDIR /workspace

COPY package.json package-lock.json ./
RUN npm ci

COPY . .

CMD ["npx", "hereby", "runtests-parallel"]
