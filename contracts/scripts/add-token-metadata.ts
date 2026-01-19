/**
 * BREACH Token Metadata Script
 * Adds on-chain metadata to the $BREACH token using Metaplex
 * 
 * Usage: cd contracts/tests && pnpm exec ts-node ../scripts/add-token-metadata.ts
 */

import {
  Connection,
  Keypair,
  PublicKey,
  Transaction,
  TransactionInstruction,
  sendAndConfirmTransaction,
} from "@solana/web3.js";
import * as fs from "fs";
import * as path from "path";

// Metaplex Token Metadata Program ID
const TOKEN_METADATA_PROGRAM_ID = new PublicKey(
  "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s"
);

// Configuration
const NETWORK = process.env.NETWORK || "devnet";
const RPC_URL =
  NETWORK === "mainnet"
    ? "https://api.mainnet-beta.solana.com"
    : "https://api.devnet.solana.com";

// Token Metadata
const TOKEN_METADATA = {
  name: "BREACH",
  symbol: "BREACH",
  uri: "https://breach-jade.vercel.app/api/token-metadata",
  sellerFeeBasisPoints: 0,
};

// Build CreateMetadataAccountV3 instruction manually
function createMetadataAccountV3Instruction(
  metadata: PublicKey,
  mint: PublicKey,
  mintAuthority: PublicKey,
  payer: PublicKey,
  updateAuthority: PublicKey,
  name: string,
  symbol: string,
  uri: string
): TransactionInstruction {
  // Instruction discriminator for CreateMetadataAccountV3
  const discriminator = Buffer.from([33]);

  // Serialize name (4-byte length prefix + string bytes)
  const nameBuffer = Buffer.alloc(4 + name.length);
  nameBuffer.writeUInt32LE(name.length, 0);
  nameBuffer.write(name, 4);

  // Serialize symbol
  const symbolBuffer = Buffer.alloc(4 + symbol.length);
  symbolBuffer.writeUInt32LE(symbol.length, 0);
  symbolBuffer.write(symbol, 4);

  // Serialize URI
  const uriBuffer = Buffer.alloc(4 + uri.length);
  uriBuffer.writeUInt32LE(uri.length, 0);
  uriBuffer.write(uri, 4);

  // Seller fee basis points (u16)
  const sellerFeeBps = Buffer.alloc(2);
  sellerFeeBps.writeUInt16LE(0, 0);

  // Creators (Option<Vec<Creator>>) - None = 0
  const creatorsOption = Buffer.from([0]);

  // Collection (Option<Collection>) - None = 0
  const collectionOption = Buffer.from([0]);

  // Uses (Option<Uses>) - None = 0
  const usesOption = Buffer.from([0]);

  // Is mutable (bool)
  const isMutable = Buffer.from([1]);

  // Collection details (Option<CollectionDetails>) - None = 0
  const collectionDetails = Buffer.from([0]);

  const data = Buffer.concat([
    discriminator,
    nameBuffer,
    symbolBuffer,
    uriBuffer,
    sellerFeeBps,
    creatorsOption,
    collectionOption,
    usesOption,
    isMutable,
    collectionDetails,
  ]);

  return new TransactionInstruction({
    keys: [
      { pubkey: metadata, isSigner: false, isWritable: true },
      { pubkey: mint, isSigner: false, isWritable: false },
      { pubkey: mintAuthority, isSigner: true, isWritable: false },
      { pubkey: payer, isSigner: true, isWritable: true },
      { pubkey: updateAuthority, isSigner: false, isWritable: false },
      {
        pubkey: new PublicKey("11111111111111111111111111111111"),
        isSigner: false,
        isWritable: false,
      },
      {
        pubkey: new PublicKey("SysvarRent111111111111111111111111111111111"),
        isSigner: false,
        isWritable: false,
      },
    ],
    programId: TOKEN_METADATA_PROGRAM_ID,
    data,
  });
}

async function main() {
  console.log(
    "╔════════════════════════════════════════════════════════════════╗"
  );
  console.log(
    "║           BREACH Token Metadata Setup                          ║"
  );
  console.log(
    "╚════════════════════════════════════════════════════════════════╝"
  );
  console.log(`\nNetwork: ${NETWORK}`);

  // Load wallet
  const walletPath = path.resolve(
    process.env.HOME || "~",
    ".config/solana/mainnet-deploy-wallet.json"
  );

  if (!fs.existsSync(walletPath)) {
    console.error(`❌ Wallet not found at ${walletPath}`);
    process.exit(1);
  }

  const secretKey = JSON.parse(fs.readFileSync(walletPath, "utf-8"));
  const payer = Keypair.fromSecretKey(Uint8Array.from(secretKey));
  console.log(`Payer: ${payer.publicKey.toBase58()}`);

  // Load token info
  const tokenInfoPath = path.resolve(
    __dirname,
    "../target/deploy/breach-token-info.json"
  );

  if (!fs.existsSync(tokenInfoPath)) {
    console.error(`❌ Token info not found at ${tokenInfoPath}`);
    console.error("Please run create-breach-token.sh first");
    process.exit(1);
  }

  const tokenInfo = JSON.parse(fs.readFileSync(tokenInfoPath, "utf-8"));
  const mintAddress = new PublicKey(tokenInfo.mintAddress);
  console.log(`Token Mint: ${mintAddress.toBase58()}`);

  // Connect to network
  const connection = new Connection(RPC_URL, "confirmed");
  const balance = await connection.getBalance(payer.publicKey);
  console.log(`Balance: ${balance / 1e9} SOL`);

  // Derive metadata PDA
  const [metadataPDA] = PublicKey.findProgramAddressSync(
    [
      Buffer.from("metadata"),
      TOKEN_METADATA_PROGRAM_ID.toBuffer(),
      mintAddress.toBuffer(),
    ],
    TOKEN_METADATA_PROGRAM_ID
  );
  console.log(`Metadata PDA: ${metadataPDA.toBase58()}`);

  // Check if metadata already exists
  const existingMetadata = await connection.getAccountInfo(metadataPDA);
  if (existingMetadata) {
    console.log("\n⚠️  Metadata already exists for this token");
    console.log("Skipping metadata creation");
    return;
  }

  // Create metadata instruction
  console.log("\n📝 Creating token metadata...");

  const createMetadataInstruction = createMetadataAccountV3Instruction(
    metadataPDA,
    mintAddress,
    payer.publicKey,
    payer.publicKey,
    payer.publicKey,
    TOKEN_METADATA.name,
    TOKEN_METADATA.symbol,
    TOKEN_METADATA.uri
  );

  // Send transaction
  const transaction = new Transaction().add(createMetadataInstruction);

  try {
    const signature = await sendAndConfirmTransaction(connection, transaction, [
      payer,
    ]);
    console.log(`\n✅ Metadata created successfully!`);
    console.log(`Signature: ${signature}`);
    console.log(
      `\nExplorer: https://explorer.solana.com/tx/${signature}?cluster=${NETWORK}`
    );

    // Update token info file
    tokenInfo.metadataPDA = metadataPDA.toBase58();
    tokenInfo.metadata = TOKEN_METADATA;
    fs.writeFileSync(tokenInfoPath, JSON.stringify(tokenInfo, null, 2));
    console.log(`\nUpdated token info at ${tokenInfoPath}`);
  } catch (error) {
    console.error("\n❌ Failed to create metadata:", error);
    process.exit(1);
  }
}

main().catch(console.error);
