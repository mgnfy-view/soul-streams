import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { assert } from "chai";
import { SoulStreams } from "../target/types/soul_streams";

import { setup } from "./utils/setup";
import { seedStrings } from "./utils/constants";
import { getStreamCountPublicKey } from "./utils/utils";

describe("Soul Streams", () => {
    let program: Program<SoulStreams>;

    before(async () => {
        ({ program } = await setup());
    });

    it("Is initialized and emits event", async () => {
        const expectedStreamCountValue = 1;

        // Listen for the initialized event
        const initializedEventListener = program.addEventListener("initialized", (event) => {
            const streamCountValue = event.streamCount.toNumber();

            console.log(`\tInitialization emitted event with value: ${streamCountValue}`);
            assert.equal(streamCountValue, expectedStreamCountValue);
        });

        await program.methods.initialize().rpc();

        // Retrieve the stream count pda
        const streamCountPublicKey = getStreamCountPublicKey(program.programId);
        const streamCountAccount = await program.account.streamCount.fetch(streamCountPublicKey);

        // Check if the stream count account has count value set to 0
        const streamCountValue = streamCountAccount.count.toNumber();
        console.log(`\tStream count pda was initialized with count value: ${streamCountValue}`);
        assert.equal(streamCountValue, expectedStreamCountValue);

        program.removeEventListener(initializedEventListener);
    });
});
