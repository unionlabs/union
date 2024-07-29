<script lang="ts">
    import SpinnerSvg from "$lib/components/spinner-svg.svelte";
    import { createClient, type Provider } from "@supabase/supabase-js";

    const apiKey =
        "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZSIsInJlZiI6ImJmZmNvbHdjYWtxcmhsem55am5zIiwicm9sZSI6ImFub24iLCJpYXQiOjE3MjIwOTc5OTgsImV4cCI6MjAzNzY3Mzk5OH0.9dVeafP8atsYqwdtPVYmzIhqMr_DEkHKdfoN3eqxjC0";
    const supabase = createClient(
        "https://bffcolwcakqrhlznyjns.supabase.co",
        apiKey,
    );

    let messages: string[] = [];
    let queuePosition: null | number = null;

    const addMessage = (message: string) => {
        messages = [...messages, message];
    };

    const addSpinMessage = (message: string, pop: number) => {
        messages = [...messages.slice(0, messages.length - pop), message];
    };

    const sleep = (ms: number) => new Promise((r) => setTimeout(r, ms));

    const onConnect = async (provider: Provider) => {
        addMessage("Session found, checking if it's our round...");
        const { data, error } = await supabase.auth.signInWithOAuth({
            provider: provider,
            options: {
                redirectTo: location.href.replace(location.hash, ""),
            },
        });
        console.log(error);
        console.log(data);
    };

    supabase.auth.onAuthStateChange((event, session) => {
        console.log(event, session);
    });

    supabase.auth
        .getSession()
        .catch(async () => {
            addMessage("Session not found, logging in...");
            onConnect("github");
            return { data: { session: null } };
        })
        .then(async ({ data: { session } }) => {
            if (session !== null) {
                while (true) {
                    {
                        const { data: queue } = await supabase
                            .from("current_queue_position")
                            .select("position");
                        console.log(queue);
                        if (queue != null && queue.length == 1) {
                            queuePosition = queue[0].position;
                        }
                    }
                    {
                        const { data: submitted } = await supabase
                            .from("contribution_submitted")
                            .select("id, created_at")
                            .filter("id", "eq", session.user.id);
                        if (submitted != null && submitted.length == 1) {
                            addMessage(
                                "You contribution has been registered at: " +
                                    submitted[0].created_at,
                            );
                            {
                                const { data: contribution } = await supabase
                                    .from("contribution")
                                    .select("created_at,success")
                                    .filter("id", "eq", session.user.id);
                                if (
                                    contribution != null &&
                                    contribution.length == 1
                                ) {
                                    const { created_at, success } =
                                        contribution[0];
                                    addMessage(
                                        "Contribution verified, valid: " +
                                            success +
                                            ", at: " +
                                            created_at,
                                    );
                                } else {
                                    addMessage(
                                        "Contribution not verified yet.",
                                    );
                                }
                            }
                            break;
                        } else {
                            addMessage("You've not contributed yet...");
                        }
                    }
                    addMessage("Session found, checking if it's our round...");
                    const {
                        data: [{ id }],
                    } = await supabase
                        .from("current_contributor_id")
                        .select("*");
                    if (id == session.user.id) {
                        addMessage(
                            "Finally us, dispatching to CLI, be patient!",
                        );
                        console.log("user: ", session.user);
                        console.log("session: ", session);
                        const {
                            data: [{ payload_id }],
                        } = await supabase
                            .from("queue")
                            .select("payload_id")
                            .filter("id", "eq", session.user.id);
                        const contributeRequest = {
                            apiKey,
                            bucket: "contributions",
                            contributorId: session.user.id,
                            jwt: session?.access_token,
                            payloadId: payload_id,
                        };
                        const success = await fetch(
                            "http://localhost:4919/contribute",
                            {
                                method: "POST",
                                body: JSON.stringify(contributeRequest),
                                headers: {
                                    "Content-Type": "application/json",
                                },
                            },
                            600 * 1000,
                        )
                            .then((response) => {
                                if (response.status == 200) {
                                    return true;
                                } else {
                                    return false;
                                }
                            })
                            .catch((e) => {
                                console.log(e);
                                addMessage(
                                    "/!\\ Could not contact CLI endpoint, make sure it is up /!\\",
                                );
                                return false;
                            });
                        if (success) {
                            addMessage("Contribution succeeded!");
                        } else {
                            for (let i = 10; i > 0; i--) {
                                addSpinMessage(
                                    "Retrying in " + i + " seconds...",
                                    i == 10 ? 0 : 1,
                                );
                                await sleep(1_000);
                            }
                        }
                    } else {
                        addMessage("Not our turn yet...");
                        for (let i = 10; i > 0; i--) {
                            addSpinMessage(
                                "Checking again in " + i + " seconds...",
                                i == 10 ? 0 : 1,
                            );
                            await sleep(1_000);
                        }
                    }
                }
            } else {
                onConnect("github");
            }
        });
</script>

<svelte:head>
    <title>Union | Ceremony</title>
    <script src="https://cdn.jsdelivr.net/npm/@supabase/supabase-js@2"></script>
</svelte:head>

<div class="w-full flex flex-col items-center">
    {#if queuePosition != null}
        Queue position: {queuePosition}
    {:else}
        <div class="flex items-center">
            Queue position: <SpinnerSvg className="w-4 h-4" />
        </div>
    {/if}

    {#each messages as message}
        <div class="flex items-center">
            {@html message}
        </div>
    {/each}
</div>
