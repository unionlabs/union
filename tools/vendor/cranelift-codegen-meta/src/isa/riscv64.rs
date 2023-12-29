use crate::cdsl::isa::TargetIsa;
use crate::cdsl::settings::{SettingGroup, SettingGroupBuilder};

use crate::shared::Definitions as SharedDefinitions;

fn define_settings(_shared: &SettingGroup) -> SettingGroup {
    let mut setting = SettingGroupBuilder::new("riscv64");

    let _has_m = setting.add_bool("has_m", "has extension M?", "", false);
    let _has_a = setting.add_bool("has_a", "has extension A?", "", false);
    let _has_f = setting.add_bool("has_f", "has extension F?", "", false);
    let _has_d = setting.add_bool("has_d", "has extension D?", "", false);
    let _has_v = setting.add_bool("has_v", "has extension V?", "", false);
    let _has_b = setting.add_bool("has_b", "has extension B?", "", false);
    let _has_c = setting.add_bool("has_c", "has extension C?", "", false);
    let _has_zbkb = setting.add_bool("has_zbkb", "has extension zbkb?", "", false);

    let _has_zicsr = setting.add_bool("has_zicsr", "has extension zicsr?", "", false);
    let _has_zifencei = setting.add_bool("has_zifencei", "has extension zifencei?", "", false);

    setting.build()
}

pub(crate) fn define(shared_defs: &mut SharedDefinitions) -> TargetIsa {
    let settings = define_settings(&shared_defs.settings);
    TargetIsa::new("riscv64", settings)
}
