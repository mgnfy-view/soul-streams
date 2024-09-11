import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { createMint, createAssociatedTokenAccount } from "@solana/spl-token";
import { SoulStreams } from "../../target/types/soul_streams";

import { Setup } from "./types";
import { DECIMALS } from "./constants";

async function setup(): Promise<Setup> {
    const provider = anchor.AnchorProvider.env();
    anchor.setProvider(provider);

    const program = anchor.workspace.SoulStreams as Program<SoulStreams>;

    const owner = (provider.wallet as anchor.Wallet).payer;
    const recipient = anchor.web3.Keypair.generate();

    const mint = await createMint(provider.connection, owner, owner.publicKey, owner.publicKey, DECIMALS);

    const ownerAssociatedTokenAccount = await createAssociatedTokenAccount(
        provider.connection,
        owner,
        mint,
        owner.publicKey
    );
    const recipientAssociatedTokenAccount = await createAssociatedTokenAccount(
        provider.connection,
        owner,
        mint,
        recipient.publicKey
    );

    return {
        provider,
        program,
        owner,
        recipient,
        mint,
        ownerAssociatedTokenAccount,
        recipientAssociatedTokenAccount,
    };
}

export { setup };
