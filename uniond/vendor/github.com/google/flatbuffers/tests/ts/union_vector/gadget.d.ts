import { FallingTub } from './falling-tub.js';
import { HandFan } from './hand-fan.js';
export declare enum Gadget {
    NONE = 0,
    FallingTub = 1,
    HandFan = 2
}
export declare function unionToGadget(type: Gadget, accessor: (obj: FallingTub | HandFan) => FallingTub | HandFan | null): FallingTub | HandFan | null;
export declare function unionListToGadget(type: Gadget, accessor: (index: number, obj: FallingTub | HandFan) => FallingTub | HandFan | null, index: number): FallingTub | HandFan | null;
