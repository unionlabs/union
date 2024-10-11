package appmodulev2

import (
	"context"

	"cosmossdk.io/core/registry"
	"cosmossdk.io/core/transaction"
)

// AppModule is a tag interface for app module implementations to use as a basis
// for extension interfaces. It provides no functionality itself, but is the
// type that all valid app modules should provide so that they can be identified
// by other modules (usually via depinject) as app modules.
type AppModule interface {
	// IsAppModule is a dummy method to tag a struct as implementing an AppModule.
	IsAppModule()

	// IsOnePerModuleType is a dummy method to help depinject resolve modules.
	IsOnePerModuleType()
}

// HasPreBlocker is the extension interface that modules should implement to run
// custom logic before BeginBlock.
type HasPreBlocker interface {
	AppModule
	// PreBlock is method that will be run before BeginBlock.
	PreBlock(context.Context) error
}

// HasBeginBlocker is the extension interface that modules should implement to run
// custom logic before transaction processing in a block.
type HasBeginBlocker interface {
	AppModule

	// BeginBlock is a method that will be run before transactions are processed in
	// a block.
	BeginBlock(context.Context) error
}

// HasEndBlocker is the extension interface that modules should implement to run
// custom logic after transaction processing in a block.
type HasEndBlocker interface {
	AppModule

	// EndBlock is a method that will be run after transactions are processed in
	// a block.
	EndBlock(context.Context) error
}

// HasTxValidator is the extension interface that modules should implement to run
// custom logic for validating transactions.
// It was previously known as AnteHandler/Decorator.
type HasTxValidator[T transaction.Tx] interface {
	AppModule

	// TxValidator is a method that will be run on each transaction.
	// If an error is returned:
	// 	                          ,---.
	//                           /    |
	//                          /     |
	//  You shall not pass!    /      |
	//                        /       |
	//           \       ___,'        |
	//                 <  -'          :
	//                  `-.__..--'``-,_\_
	//                     |o/ <o>` :,.)_`>
	//                     :/ `     ||/)
	//                     (_.).__,-` |\
	//                     /( `.``   `| :
	//                     \'`-.)  `  ; ;
	//                     | `       /-<
	//                     |     `  /   `.
	//     ,-_-..____     /|  `    :__..-'\
	//    /,'-.__\\  ``-./ :`      ;       \
	//    `\ `\  `\\  \ :  (   `  /  ,   `. \
	//      \` \   \\   |  | `   :  :     .\ \
	//       \ `\_  ))  :  ;     |  |      ): :
	//      (`-.-'\ ||  |\ \   ` ;  ;       | |
	//       \-_   `;;._   ( `  /  /_       | |
	//        `-.-.// ,'`-._\__/_,'         ; |
	//           \:: :     /     `     ,   /  |
	//            || |    (        ,' /   /   |
	//            ||                ,'   /    |
	TxValidator(ctx context.Context, tx T) error
}

// HasUpdateValidators is an extension interface that contains information about the AppModule and UpdateValidators.
// It can be seen as the alternative of the Cosmos SDK' HasABCIEndBlocker.
// Both are still supported.
type HasUpdateValidators interface {
	AppModule

	UpdateValidators(ctx context.Context) ([]ValidatorUpdate, error)
}

// ValidatorUpdate defines a validator update.
type ValidatorUpdate struct {
	PubKey     []byte
	PubKeyType string
	Power      int64 // updated power of the validator
}

// HasRegisterInterfaces is the interface for modules to register their msg types.
type HasRegisterInterfaces interface {
	RegisterInterfaces(registry.InterfaceRegistrar)
}
