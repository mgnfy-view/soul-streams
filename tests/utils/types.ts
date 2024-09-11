import * as anchor from "@coral-xyz/anchor";
import { SoulStreams } from "../../target/types/soul_streams";

interface Setup {
    provider: anchor.AnchorProvider;
    program: anchor.Program<SoulStreams>;
    owner: anchor.web3.Keypair;
    recipient: anchor.web3.Keypair;
    mint: anchor.web3.PublicKey;
    ownerAssociatedTokenAccount: anchor.web3.PublicKey;
    recipientAssociatedTokenAccount: anchor.web3.PublicKey;
}

interface SeedStrings {
    streamCount: string;
    stream: string;
    tokenAccount: string;
}

export { Setup, SeedStrings };
