#[cfg(feature = "Devices_Sensors_Custom")]
pub mod Custom;
pub type Accelerometer = *mut ::core::ffi::c_void;
pub type AccelerometerDataThreshold = *mut ::core::ffi::c_void;
pub type AccelerometerReading = *mut ::core::ffi::c_void;
pub type AccelerometerReadingChangedEventArgs = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Devices_Sensors\"`*"]
#[repr(transparent)]
pub struct AccelerometerReadingType(pub i32);
impl AccelerometerReadingType {
    pub const Standard: Self = Self(0i32);
    pub const Linear: Self = Self(1i32);
    pub const Gravity: Self = Self(2i32);
}
impl ::core::marker::Copy for AccelerometerReadingType {}
impl ::core::clone::Clone for AccelerometerReadingType {
    fn clone(&self) -> Self {
        *self
    }
}
pub type AccelerometerShakenEventArgs = *mut ::core::ffi::c_void;
pub type ActivitySensor = *mut ::core::ffi::c_void;
pub type ActivitySensorReading = *mut ::core::ffi::c_void;
pub type ActivitySensorReadingChangeReport = *mut ::core::ffi::c_void;
pub type ActivitySensorReadingChangedEventArgs = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Devices_Sensors\"`*"]
#[repr(transparent)]
pub struct ActivitySensorReadingConfidence(pub i32);
impl ActivitySensorReadingConfidence {
    pub const High: Self = Self(0i32);
    pub const Low: Self = Self(1i32);
}
impl ::core::marker::Copy for ActivitySensorReadingConfidence {}
impl ::core::clone::Clone for ActivitySensorReadingConfidence {
    fn clone(&self) -> Self {
        *self
    }
}
pub type ActivitySensorTriggerDetails = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Devices_Sensors\"`*"]
#[repr(transparent)]
pub struct ActivityType(pub i32);
impl ActivityType {
    pub const Unknown: Self = Self(0i32);
    pub const Idle: Self = Self(1i32);
    pub const Stationary: Self = Self(2i32);
    pub const Fidgeting: Self = Self(3i32);
    pub const Walking: Self = Self(4i32);
    pub const Running: Self = Self(5i32);
    pub const InVehicle: Self = Self(6i32);
    pub const Biking: Self = Self(7i32);
}
impl ::core::marker::Copy for ActivityType {}
impl ::core::clone::Clone for ActivityType {
    fn clone(&self) -> Self {
        *self
    }
}
pub type Altimeter = *mut ::core::ffi::c_void;
pub type AltimeterReading = *mut ::core::ffi::c_void;
pub type AltimeterReadingChangedEventArgs = *mut ::core::ffi::c_void;
pub type Barometer = *mut ::core::ffi::c_void;
pub type BarometerDataThreshold = *mut ::core::ffi::c_void;
pub type BarometerReading = *mut ::core::ffi::c_void;
pub type BarometerReadingChangedEventArgs = *mut ::core::ffi::c_void;
pub type Compass = *mut ::core::ffi::c_void;
pub type CompassDataThreshold = *mut ::core::ffi::c_void;
pub type CompassReading = *mut ::core::ffi::c_void;
pub type CompassReadingChangedEventArgs = *mut ::core::ffi::c_void;
pub type Gyrometer = *mut ::core::ffi::c_void;
pub type GyrometerDataThreshold = *mut ::core::ffi::c_void;
pub type GyrometerReading = *mut ::core::ffi::c_void;
pub type GyrometerReadingChangedEventArgs = *mut ::core::ffi::c_void;
pub type HingeAngleReading = *mut ::core::ffi::c_void;
pub type HingeAngleSensor = *mut ::core::ffi::c_void;
pub type HingeAngleSensorReadingChangedEventArgs = *mut ::core::ffi::c_void;
pub type ISensorDataThreshold = *mut ::core::ffi::c_void;
pub type Inclinometer = *mut ::core::ffi::c_void;
pub type InclinometerDataThreshold = *mut ::core::ffi::c_void;
pub type InclinometerReading = *mut ::core::ffi::c_void;
pub type InclinometerReadingChangedEventArgs = *mut ::core::ffi::c_void;
pub type LightSensor = *mut ::core::ffi::c_void;
pub type LightSensorDataThreshold = *mut ::core::ffi::c_void;
pub type LightSensorReading = *mut ::core::ffi::c_void;
pub type LightSensorReadingChangedEventArgs = *mut ::core::ffi::c_void;
pub type Magnetometer = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Devices_Sensors\"`*"]
#[repr(transparent)]
pub struct MagnetometerAccuracy(pub i32);
impl MagnetometerAccuracy {
    pub const Unknown: Self = Self(0i32);
    pub const Unreliable: Self = Self(1i32);
    pub const Approximate: Self = Self(2i32);
    pub const High: Self = Self(3i32);
}
impl ::core::marker::Copy for MagnetometerAccuracy {}
impl ::core::clone::Clone for MagnetometerAccuracy {
    fn clone(&self) -> Self {
        *self
    }
}
pub type MagnetometerDataThreshold = *mut ::core::ffi::c_void;
pub type MagnetometerReading = *mut ::core::ffi::c_void;
pub type MagnetometerReadingChangedEventArgs = *mut ::core::ffi::c_void;
pub type OrientationSensor = *mut ::core::ffi::c_void;
pub type OrientationSensorReading = *mut ::core::ffi::c_void;
pub type OrientationSensorReadingChangedEventArgs = *mut ::core::ffi::c_void;
pub type Pedometer = *mut ::core::ffi::c_void;
pub type PedometerDataThreshold = *mut ::core::ffi::c_void;
pub type PedometerReading = *mut ::core::ffi::c_void;
pub type PedometerReadingChangedEventArgs = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Devices_Sensors\"`*"]
#[repr(transparent)]
pub struct PedometerStepKind(pub i32);
impl PedometerStepKind {
    pub const Unknown: Self = Self(0i32);
    pub const Walking: Self = Self(1i32);
    pub const Running: Self = Self(2i32);
}
impl ::core::marker::Copy for PedometerStepKind {}
impl ::core::clone::Clone for PedometerStepKind {
    fn clone(&self) -> Self {
        *self
    }
}
pub type ProximitySensor = *mut ::core::ffi::c_void;
pub type ProximitySensorDataThreshold = *mut ::core::ffi::c_void;
pub type ProximitySensorDisplayOnOffController = *mut ::core::ffi::c_void;
pub type ProximitySensorReading = *mut ::core::ffi::c_void;
pub type ProximitySensorReadingChangedEventArgs = *mut ::core::ffi::c_void;
pub type SensorDataThresholdTriggerDetails = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Devices_Sensors\"`*"]
#[repr(transparent)]
pub struct SensorOptimizationGoal(pub i32);
impl SensorOptimizationGoal {
    pub const Precision: Self = Self(0i32);
    pub const PowerEfficiency: Self = Self(1i32);
}
impl ::core::marker::Copy for SensorOptimizationGoal {}
impl ::core::clone::Clone for SensorOptimizationGoal {
    fn clone(&self) -> Self {
        *self
    }
}
pub type SensorQuaternion = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Devices_Sensors\"`*"]
#[repr(transparent)]
pub struct SensorReadingType(pub i32);
impl SensorReadingType {
    pub const Absolute: Self = Self(0i32);
    pub const Relative: Self = Self(1i32);
}
impl ::core::marker::Copy for SensorReadingType {}
impl ::core::clone::Clone for SensorReadingType {
    fn clone(&self) -> Self {
        *self
    }
}
pub type SensorRotationMatrix = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Devices_Sensors\"`*"]
#[repr(transparent)]
pub struct SensorType(pub i32);
impl SensorType {
    pub const Accelerometer: Self = Self(0i32);
    pub const ActivitySensor: Self = Self(1i32);
    pub const Barometer: Self = Self(2i32);
    pub const Compass: Self = Self(3i32);
    pub const CustomSensor: Self = Self(4i32);
    pub const Gyroscope: Self = Self(5i32);
    pub const ProximitySensor: Self = Self(6i32);
    pub const Inclinometer: Self = Self(7i32);
    pub const LightSensor: Self = Self(8i32);
    pub const OrientationSensor: Self = Self(9i32);
    pub const Pedometer: Self = Self(10i32);
    pub const RelativeInclinometer: Self = Self(11i32);
    pub const RelativeOrientationSensor: Self = Self(12i32);
    pub const SimpleOrientationSensor: Self = Self(13i32);
}
impl ::core::marker::Copy for SensorType {}
impl ::core::clone::Clone for SensorType {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Devices_Sensors\"`*"]
#[repr(transparent)]
pub struct SimpleOrientation(pub i32);
impl SimpleOrientation {
    pub const NotRotated: Self = Self(0i32);
    pub const Rotated90DegreesCounterclockwise: Self = Self(1i32);
    pub const Rotated180DegreesCounterclockwise: Self = Self(2i32);
    pub const Rotated270DegreesCounterclockwise: Self = Self(3i32);
    pub const Faceup: Self = Self(4i32);
    pub const Facedown: Self = Self(5i32);
}
impl ::core::marker::Copy for SimpleOrientation {}
impl ::core::clone::Clone for SimpleOrientation {
    fn clone(&self) -> Self {
        *self
    }
}
pub type SimpleOrientationSensor = *mut ::core::ffi::c_void;
pub type SimpleOrientationSensorOrientationChangedEventArgs = *mut ::core::ffi::c_void;
