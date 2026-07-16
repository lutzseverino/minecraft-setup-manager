import { PublicKey, Signature } from "@threema/wasm-minisign-verify";

export function verifyUpdateSignature(
  bytes,
  encodedSignature,
  encodedPublicKey,
) {
  const publicKey = PublicKey.decode(
    Buffer.from(encodedPublicKey, "base64").toString("utf8"),
  );
  let signature;

  try {
    signature = Signature.decode(
      Buffer.from(encodedSignature, "base64").toString("utf8"),
    );
    if (!publicKey.verify(bytes, signature)) {
      throw new Error("Minisign verification returned false.");
    }
  } finally {
    signature?.free();
    publicKey.free();
  }
}
