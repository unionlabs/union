/*
Package evidence implements a Cosmos SDK module, per ADR 009, that allows for the
submission and handling of arbitrary evidence of misbehavior.

All concrete evidence types must implement the Evidence interface contract. Submitted
evidence is first routed through the evidence module's Router in which it attempts
to find a corresponding Handler for that specific evidence type. Each evidence type
must have a Handler registered with the evidence module's keeper in order for it
to be successfully executed.

Each corresponding handler must also fulfill the Handler interface contract. The
Handler for a given Evidence type can perform any arbitrary state transitions
such as slashing, jailing, and tombstoning. This provides developers with great
flexibility in designing evidence handling.

A full setup of the evidence module may look something as follows:

	// First, create the keeper
	evidenceKeeper := evidence.NewKeeper(
	  appCodec, runtime.NewKVStoreService(keys[evidencetypes.StoreKey]),
	  &app.StakingKeeper, app.SlashingKeeper,
	)

	// Second, create the evidence Handler and register all desired routes.
	evidenceRouter := evidence.NewRouter().
	  AddRoute(evidenceRoute, evidenceHandler).
	  AddRoute(..., ...)

	evidenceKeeper.SetRouter(evidenceRouter)

	app.EvidenceKeeper = *evidenceKeeper

	app.mm = module.NewManager(
	  // ...
	  evidence.NewAppModule(app.EvidenceKeeper),
	)

	// Remaining application bootstrapping...
*/
package evidence
