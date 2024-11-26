#[macro_export]
macro_rules! core_run {
    ($core_instance:ident) => {
        // Unpack all the peripherals
        let serial = $core_instance.bsp.serial.replace(None).unwrap();
        let can = $core_instance.bsp.can.replace(None).unwrap();

        // Create Channels
        static SERIAL_CHANNEL: CanChannel = CanChannel::new();
        static CAN_CHANNEL: CanChannel = CanChannel::new();

        // Spawn tasks
        $core_instance
            .spawner
            .spawn(slcan_task(
                serial,
                SERIAL_CHANNEL.receiver(),
                CAN_CHANNEL.sender(),
            ))
            .unwrap();
        $core_instance
            .spawner
            .spawn(can_task(
                can,
                CAN_CHANNEL.receiver(),
                SERIAL_CHANNEL.sender(),
            ))
            .unwrap();
    };
}

#[macro_export]
macro_rules! core_create_tasks {
    ($SerialType:ty, $CanType:ty) => {
        #[embassy_executor::task]
        async fn slcan_task(
            serial: $SerialType,
            channel_in: CanChannelReceiver,
            channel_out: CanChannelSender,
        ) {
            Core::<$CanType, $SerialType>::slcan_task(serial, channel_in, channel_out).await;
        }

        #[embassy_executor::task]
        async fn can_task(
            can: $CanType,
            channel_in: CanChannelReceiver,
            channel_out: CanChannelSender,
        ) {
            Core::<$CanType, $SerialType>::can_task(can, channel_in, channel_out).await;
        }
    };
}
