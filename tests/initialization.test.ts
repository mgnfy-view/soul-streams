import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { SoulStreams } from "../target/types/soul_streams";

describe("Soul Streams", () => {
    const provider = anchor.AnchorProvider.env();
    anchor.setProvider(provider);

    const program = anchor.workspace.SoulStreams as Program<SoulStreams>;

    it("Is initialized!", async () => {
        await program.methods.initialize().rpc();
    });
});
