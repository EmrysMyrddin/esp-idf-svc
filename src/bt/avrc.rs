pub mod controller {
    use core::{
        borrow::Borrow,
        convert::{TryFrom, TryInto},
        fmt::{self, Debug},
        marker::PhantomData,
        sync::atomic::{AtomicBool, Ordering},
    };

    use esp_idf_sys::*;

    use log::info;

    use num_enum::TryFromPrimitive;

    use crate::bt::{BdAddr, BtCallback, BtClassicEnabled, BtDriver};

    #[derive(Debug, Copy, Clone, Eq, PartialEq, TryFromPrimitive)]
    #[repr(u8)]
    pub enum KeyCode {
        Select = esp_avrc_pt_cmd_t_ESP_AVRC_PT_CMD_SELECT as _,
        Up = esp_avrc_pt_cmd_t_ESP_AVRC_PT_CMD_UP as _,
        Down = esp_avrc_pt_cmd_t_ESP_AVRC_PT_CMD_DOWN as _,
        Left = esp_avrc_pt_cmd_t_ESP_AVRC_PT_CMD_LEFT as _,
        Right = esp_avrc_pt_cmd_t_ESP_AVRC_PT_CMD_RIGHT as _,
        RightUp = esp_avrc_pt_cmd_t_ESP_AVRC_PT_CMD_RIGHT_UP as _,
        RightDown = esp_avrc_pt_cmd_t_ESP_AVRC_PT_CMD_RIGHT_DOWN as _,
        LeftUp = esp_avrc_pt_cmd_t_ESP_AVRC_PT_CMD_LEFT_UP as _,
        LeftDown = esp_avrc_pt_cmd_t_ESP_AVRC_PT_CMD_LEFT_DOWN as _,
        RootMenu = esp_avrc_pt_cmd_t_ESP_AVRC_PT_CMD_ROOT_MENU as _,
        SetupMenu = esp_avrc_pt_cmd_t_ESP_AVRC_PT_CMD_SETUP_MENU as _,
        ContentsMenu = esp_avrc_pt_cmd_t_ESP_AVRC_PT_CMD_CONT_MENU as _,
        FavMenu = esp_avrc_pt_cmd_t_ESP_AVRC_PT_CMD_FAV_MENU as _,
        Exit = esp_avrc_pt_cmd_t_ESP_AVRC_PT_CMD_EXIT as _,
        Num0 = esp_avrc_pt_cmd_t_ESP_AVRC_PT_CMD_0 as _,
        Num1 = esp_avrc_pt_cmd_t_ESP_AVRC_PT_CMD_1 as _,
        Num2 = esp_avrc_pt_cmd_t_ESP_AVRC_PT_CMD_2 as _,
        Num3 = esp_avrc_pt_cmd_t_ESP_AVRC_PT_CMD_3 as _,
        Num4 = esp_avrc_pt_cmd_t_ESP_AVRC_PT_CMD_4 as _,
        Num5 = esp_avrc_pt_cmd_t_ESP_AVRC_PT_CMD_5 as _,
        Num6 = esp_avrc_pt_cmd_t_ESP_AVRC_PT_CMD_6 as _,
        Num7 = esp_avrc_pt_cmd_t_ESP_AVRC_PT_CMD_7 as _,
        Num8 = esp_avrc_pt_cmd_t_ESP_AVRC_PT_CMD_8 as _,
        Num9 = esp_avrc_pt_cmd_t_ESP_AVRC_PT_CMD_9 as _,
        Dot = esp_avrc_pt_cmd_t_ESP_AVRC_PT_CMD_DOT as _,
        Enter = esp_avrc_pt_cmd_t_ESP_AVRC_PT_CMD_ENTER as _,
        Clear = esp_avrc_pt_cmd_t_ESP_AVRC_PT_CMD_CLEAR as _,
        ChannelUp = esp_avrc_pt_cmd_t_ESP_AVRC_PT_CMD_CHAN_UP as _,
        ChannelDown = esp_avrc_pt_cmd_t_ESP_AVRC_PT_CMD_CHAN_DOWN as _,
        PreviousChannel = esp_avrc_pt_cmd_t_ESP_AVRC_PT_CMD_PREV_CHAN as _,
        SoundSelect = esp_avrc_pt_cmd_t_ESP_AVRC_PT_CMD_SOUND_SEL as _,
        InputSelect = esp_avrc_pt_cmd_t_ESP_AVRC_PT_CMD_INPUT_SEL as _,
        DisplayInformation = esp_avrc_pt_cmd_t_ESP_AVRC_PT_CMD_DISP_INFO as _,
        Help = esp_avrc_pt_cmd_t_ESP_AVRC_PT_CMD_HELP as _,
        PageUp = esp_avrc_pt_cmd_t_ESP_AVRC_PT_CMD_PAGE_UP as _,
        PageDown = esp_avrc_pt_cmd_t_ESP_AVRC_PT_CMD_PAGE_DOWN as _,
        Power = esp_avrc_pt_cmd_t_ESP_AVRC_PT_CMD_POWER as _,
        VolumeUp = esp_avrc_pt_cmd_t_ESP_AVRC_PT_CMD_VOL_UP as _,
        VolumeDown = esp_avrc_pt_cmd_t_ESP_AVRC_PT_CMD_VOL_DOWN as _,
        Mute = esp_avrc_pt_cmd_t_ESP_AVRC_PT_CMD_MUTE as _,
        Play = esp_avrc_pt_cmd_t_ESP_AVRC_PT_CMD_PLAY as _,
        Stop = esp_avrc_pt_cmd_t_ESP_AVRC_PT_CMD_STOP as _,
        Pause = esp_avrc_pt_cmd_t_ESP_AVRC_PT_CMD_PAUSE as _,
        Record = esp_avrc_pt_cmd_t_ESP_AVRC_PT_CMD_RECORD as _,
        Rewind = esp_avrc_pt_cmd_t_ESP_AVRC_PT_CMD_REWIND as _,
        FastForward = esp_avrc_pt_cmd_t_ESP_AVRC_PT_CMD_FAST_FORWARD as _,
        Eject = esp_avrc_pt_cmd_t_ESP_AVRC_PT_CMD_EJECT as _,
        Forward = esp_avrc_pt_cmd_t_ESP_AVRC_PT_CMD_FORWARD as _,
        Backward = esp_avrc_pt_cmd_t_ESP_AVRC_PT_CMD_BACKWARD as _,
        Angle = esp_avrc_pt_cmd_t_ESP_AVRC_PT_CMD_ANGLE as _,
        Subpicture = esp_avrc_pt_cmd_t_ESP_AVRC_PT_CMD_SUBPICT as _,
        F1 = esp_avrc_pt_cmd_t_ESP_AVRC_PT_CMD_F1 as _,
        F2 = esp_avrc_pt_cmd_t_ESP_AVRC_PT_CMD_F2 as _,
        F3 = esp_avrc_pt_cmd_t_ESP_AVRC_PT_CMD_F3 as _,
        F4 = esp_avrc_pt_cmd_t_ESP_AVRC_PT_CMD_F4 as _,
        F5 = esp_avrc_pt_cmd_t_ESP_AVRC_PT_CMD_F5 as _,
        Vendor = esp_avrc_pt_cmd_t_ESP_AVRC_PT_CMD_VENDOR as _,
    }

    #[derive(Debug, Copy, Clone, Eq, PartialEq, TryFromPrimitive)]
    #[repr(u32)]
    pub enum ResponseCode {
        NotImplemented = esp_avrc_rsp_t_ESP_AVRC_RSP_NOT_IMPL,
        Accepted = esp_avrc_rsp_t_ESP_AVRC_RSP_ACCEPT,
        Rejected = esp_avrc_rsp_t_ESP_AVRC_RSP_REJECT,
        InTransition = esp_avrc_rsp_t_ESP_AVRC_RSP_IN_TRANS,
        Implemented = esp_avrc_rsp_t_ESP_AVRC_RSP_IMPL_STBL,
        Changed = esp_avrc_rsp_t_ESP_AVRC_RSP_CHANGED,
        Interim = esp_avrc_rsp_t_ESP_AVRC_RSP_INTERIM,
    }

    #[derive(Debug, Copy, Clone, Eq, PartialEq, TryFromPrimitive)]
    #[repr(u32)]
    pub enum PlaybackStatus {
        Stopped = esp_avrc_playback_stat_t_ESP_AVRC_PLAYBACK_STOPPED,
        Playing = esp_avrc_playback_stat_t_ESP_AVRC_PLAYBACK_PLAYING,
        Paused = esp_avrc_playback_stat_t_ESP_AVRC_PLAYBACK_PAUSED,
        SeekForward = esp_avrc_playback_stat_t_ESP_AVRC_PLAYBACK_FWD_SEEK,
        SeekBackward = esp_avrc_playback_stat_t_ESP_AVRC_PLAYBACK_REV_SEEK,
        Error = esp_avrc_playback_stat_t_ESP_AVRC_PLAYBACK_ERROR,
    }

    #[derive(Debug, Copy, Clone, Eq, PartialEq, TryFromPrimitive)]
    #[repr(u32)]
    pub enum BatteryStatus {
        Normal = esp_avrc_batt_stat_t_ESP_AVRC_BATT_NORMAL,
        Warning = esp_avrc_batt_stat_t_ESP_AVRC_BATT_WARNING,
        Critical = esp_avrc_batt_stat_t_ESP_AVRC_BATT_CRITICAL,
        Charging = esp_avrc_batt_stat_t_ESP_AVRC_BATT_EXTERNAL,
        Charged = esp_avrc_batt_stat_t_ESP_AVRC_BATT_FULL_CHARGE,
    }

    #[derive(Debug, Copy, Clone, Eq, PartialEq, TryFromPrimitive)]
    #[repr(u8)]
    pub enum NotificationType {
        Playback = esp_avrc_rn_event_ids_t_ESP_AVRC_RN_PLAY_STATUS_CHANGE as _,
        TrackChanged = esp_avrc_rn_event_ids_t_ESP_AVRC_RN_TRACK_CHANGE as _,
        TrackEnd = esp_avrc_rn_event_ids_t_ESP_AVRC_RN_TRACK_REACHED_END as _,
        TrackStart = esp_avrc_rn_event_ids_t_ESP_AVRC_RN_TRACK_REACHED_START as _,
        PlaybackPosition = esp_avrc_rn_event_ids_t_ESP_AVRC_RN_PLAY_POS_CHANGED as _,
        BatteryStatus = esp_avrc_rn_event_ids_t_ESP_AVRC_RN_BATTERY_STATUS_CHANGE as _,
        SystemStatus = esp_avrc_rn_event_ids_t_ESP_AVRC_RN_SYSTEM_STATUS_CHANGE as _,
        AppSettings = esp_avrc_rn_event_ids_t_ESP_AVRC_RN_APP_SETTING_CHANGE as _,
        NowPlaying = esp_avrc_rn_event_ids_t_ESP_AVRC_RN_NOW_PLAYING_CHANGE as _,
        AvailablePlayers = esp_avrc_rn_event_ids_t_ESP_AVRC_RN_AVAILABLE_PLAYERS_CHANGE as _,
        AddressedPlayer = esp_avrc_rn_event_ids_t_ESP_AVRC_RN_ADDRESSED_PLAYER_CHANGE as _,
        Uuids = esp_avrc_rn_event_ids_t_ESP_AVRC_RN_UIDS_CHANGE as _,
        Volume = esp_avrc_rn_event_ids_t_ESP_AVRC_RN_VOLUME_CHANGE as _,
    }

    #[derive(Debug, Copy, Clone, Eq, PartialEq)]
    pub enum Notification {
        Volume(u8),
        Playback(PlaybackStatus),
        TrackChanged,
        TrackStarted,
        TrackEnded,
        PlaybackPosition(u32),
        Battery(BatteryStatus),
        SystemStatus,
        AppSettings,
        NowPlaying,
        AvailablePlayers,
        AddressedPlayer,
        Uuids,
        Other(u8),
    }

    // /// AVRC feature bit mask
    // typedef enum {
    //     ESP_AVRC_FEAT_RCTG = 0x0001,                 /*!< remote control target */
    //     ESP_AVRC_FEAT_RCCT = 0x0002,                 /*!< remote control controller */
    //     ESP_AVRC_FEAT_VENDOR = 0x0008,               /*!< remote control vendor dependent commands */
    //     ESP_AVRC_FEAT_BROWSE = 0x0010,               /*!< use browsing channel */
    //     ESP_AVRC_FEAT_META_DATA = 0x0040,            /*!< remote control metadata transfer command/response */
    //     ESP_AVRC_FEAT_ADV_CTRL = 0x0200,             /*!< remote control advanced control command/response */
    // } esp_avrc_features_t;

    // /// AVRC supported features flag retrieved in SDP record
    // typedef enum {
    //     ESP_AVRC_FEAT_FLAG_CAT1 = 0x0001,                             /*!< category 1 */
    //     ESP_AVRC_FEAT_FLAG_CAT2 = 0x0002,                             /*!< category 2 */
    //     ESP_AVRC_FEAT_FLAG_CAT3 = 0x0004,                             /*!< category 3 */
    //     ESP_AVRC_FEAT_FLAG_CAT4 = 0x0008,                             /*!< category 4 */
    //     ESP_AVRC_FEAT_FLAG_BROWSING = 0x0040,                         /*!< browsing */
    //     ESP_AVRC_FEAT_FLAG_COVER_ART_GET_IMAGE_PROP = 0x0080,         /*!< Cover Art GetImageProperties */
    //     ESP_AVRC_FEAT_FLAG_COVER_ART_GET_IMAGE = 0x0100,              /*!< Cover Art GetImage */
    //     ESP_AVRC_FEAT_FLAG_COVER_ART_GET_LINKED_THUMBNAIL = 0x0200,   /*!< Cover Art GetLinkedThumbnail */
    // } esp_avrc_feature_flag_t;

    #[derive(Debug, Copy, Clone, Eq, PartialEq, TryFromPrimitive)]
    #[repr(u8)]
    pub enum EqualizerMode {
        Off = esp_avrc_ps_eq_value_ids_t_ESP_AVRC_PS_EQUALIZER_OFF as _,
        On = esp_avrc_ps_eq_value_ids_t_ESP_AVRC_PS_EQUALIZER_ON as _,
    }

    #[derive(Debug, Copy, Clone, Eq, PartialEq, TryFromPrimitive)]
    #[repr(u8)]
    pub enum RepeatMode {
        Off = esp_avrc_ps_rpt_value_ids_t_ESP_AVRC_PS_REPEAT_OFF as _,
        Single = esp_avrc_ps_rpt_value_ids_t_ESP_AVRC_PS_REPEAT_SINGLE as _,
        Group = esp_avrc_ps_rpt_value_ids_t_ESP_AVRC_PS_REPEAT_GROUP as _,
    }

    #[derive(Debug, Copy, Clone, Eq, PartialEq, TryFromPrimitive)]
    #[repr(u8)]
    pub enum ShuffleMode {
        Off = esp_avrc_ps_shf_value_ids_t_ESP_AVRC_PS_SHUFFLE_OFF as _,
        All = esp_avrc_ps_shf_value_ids_t_ESP_AVRC_PS_SHUFFLE_ALL as _,
        Group = esp_avrc_ps_shf_value_ids_t_ESP_AVRC_PS_SHUFFLE_GROUP as _,
    }

    #[derive(Debug, Copy, Clone, Eq, PartialEq, TryFromPrimitive)]
    #[repr(u8)]
    pub enum ScanMode {
        Off = esp_avrc_ps_scn_value_ids_t_ESP_AVRC_PS_SCAN_OFF as _,
        All = esp_avrc_ps_scn_value_ids_t_ESP_AVRC_PS_SCAN_ALL as _,
        Group = esp_avrc_ps_scn_value_ids_t_ESP_AVRC_PS_SCAN_GROUP as _,
    }

    #[derive(Debug, Copy, Clone, Eq, PartialEq)]
    #[repr(u8)]
    pub enum PlayerAttributeId {
        EqualizerMode(EqualizerMode),
        RepeatMode(RepeatMode),
        ShuffleMode(ShuffleMode),
        ScanMode(ScanMode),
    }

    impl From<PlayerAttributeId> for (esp_avrc_ps_attr_ids_t, u8) {
        fn from(value: PlayerAttributeId) -> Self {
            match value {
                PlayerAttributeId::EqualizerMode(mode) => {
                    (esp_avrc_ps_attr_ids_t_ESP_AVRC_PS_EQUALIZER, mode as _)
                }
                PlayerAttributeId::RepeatMode(mode) => {
                    (esp_avrc_ps_attr_ids_t_ESP_AVRC_PS_REPEAT_MODE, mode as _)
                }
                PlayerAttributeId::ShuffleMode(mode) => {
                    (esp_avrc_ps_attr_ids_t_ESP_AVRC_PS_SHUFFLE_MODE, mode as _)
                }
                PlayerAttributeId::ScanMode(mode) => {
                    (esp_avrc_ps_attr_ids_t_ESP_AVRC_PS_SCAN_MODE, mode as _)
                }
            }
        }
    }

    // impl From<(esp_avrc_ps_attr_ids_t, u8)> for PlayerAttributeId {
    //     fn from(value: (esp_avrc_ps_attr_ids_t, u8)) -> Self {

    //     }
    // }

    pub struct EventRawData<'a>(pub &'a esp_avrc_ct_cb_param_t);

    impl<'a> Debug for EventRawData<'a> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_tuple("RawData").finish()
        }
    }

    #[derive(Debug)]
    pub enum AvrccEvent<'a> {
        Connected(BdAddr),
        Disconnected(BdAddr),
        Passthrough {
            transaction_level: u8,
            key_code: KeyCode,
            key_pressed: bool,
            response_code: ResponseCode,
        },
        Attribute {
            id: u8,
            text: &'a str,
        },
        PlayStatus,
        Notification(Notification),
        RemoteFeatures {
            bd_addr: BdAddr,
            features: u32,
            tg_features: u16,
        },
        Capabilities,
        Volume(u8),
        Other {
            raw_event: esp_avrc_ct_cb_event_t,
            raw_data: EventRawData<'a>,
        },
    }

    #[allow(non_upper_case_globals)]
    impl<'a> From<(esp_avrc_ct_cb_event_t, &'a esp_avrc_ct_cb_param_t)> for AvrccEvent<'a> {
        fn from(value: (esp_avrc_ct_cb_event_t, &'a esp_avrc_ct_cb_param_t)) -> Self {
            let (event, param) = value;

            unsafe {
                match event {
                    esp_avrc_ct_cb_event_t_ESP_AVRC_CT_CONNECTION_STATE_EVT => {
                        if param.conn_stat.connected {
                            Self::Connected(param.conn_stat.remote_bda.into())
                        } else {
                            Self::Disconnected(param.conn_stat.remote_bda.into())
                        }
                    }
                    esp_avrc_ct_cb_event_t_ESP_AVRC_CT_PASSTHROUGH_RSP_EVT => Self::Passthrough {
                        transaction_level: param.psth_rsp.tl,
                        key_code: param.psth_rsp.key_code.try_into().unwrap(),
                        key_pressed: param.psth_rsp.key_state == 0,
                        response_code: param.psth_rsp.rsp_code.try_into().unwrap(),
                    },
                    esp_avrc_ct_cb_event_t_ESP_AVRC_CT_METADATA_RSP_EVT => Self::Attribute {
                        id: param.meta_rsp.attr_id,
                        text: core::str::from_utf8_unchecked(core::slice::from_raw_parts(
                            param.meta_rsp.attr_text,
                            param.meta_rsp.attr_length as _,
                        )),
                    },
                    esp_avrc_ct_cb_event_t_ESP_AVRC_CT_PLAY_STATUS_RSP_EVT => Self::PlayStatus,
                    esp_avrc_ct_cb_event_t_ESP_AVRC_CT_CHANGE_NOTIFY_EVT => Self::Notification(
                        match NotificationType::try_from(param.change_ntf.event_id).unwrap() {
                            NotificationType::Playback => Notification::Playback(
                                param
                                    .change_ntf
                                    .event_parameter
                                    .playback
                                    .try_into()
                                    .unwrap(),
                            ),
                            NotificationType::TrackChanged => Notification::TrackChanged,
                            NotificationType::TrackEnd => Notification::TrackEnded,
                            NotificationType::TrackStart => Notification::TrackStarted,
                            NotificationType::PlaybackPosition => Notification::PlaybackPosition(
                                param.change_ntf.event_parameter.playback,
                            ),
                            NotificationType::BatteryStatus => Notification::Battery(
                                param.change_ntf.event_parameter.batt.try_into().unwrap(),
                            ),
                            NotificationType::SystemStatus => Notification::SystemStatus,
                            NotificationType::AppSettings => Notification::AppSettings,
                            NotificationType::NowPlaying => Notification::NowPlaying,
                            NotificationType::AvailablePlayers => Notification::AvailablePlayers,
                            NotificationType::AddressedPlayer => Notification::AddressedPlayer,
                            NotificationType::Uuids => Notification::Uuids,
                            NotificationType::Volume => {
                                Notification::Volume(param.change_ntf.event_parameter.volume)
                            }
                        },
                    ),
                    esp_avrc_ct_cb_event_t_ESP_AVRC_CT_REMOTE_FEATURES_EVT => {
                        Self::RemoteFeatures {
                            bd_addr: param.rmt_feats.remote_bda.into(),
                            features: param.rmt_feats.feat_mask, // TODO
                            tg_features: param.rmt_feats.tg_feat_flag, // TODO
                        }
                    }
                    esp_avrc_ct_cb_event_t_ESP_AVRC_CT_GET_RN_CAPABILITIES_RSP_EVT => {
                        Self::Capabilities {}
                    }
                    esp_avrc_ct_cb_event_t_ESP_AVRC_CT_SET_ABSOLUTE_VOLUME_RSP_EVT => {
                        Self::Volume(param.set_volume_rsp.volume)
                    }
                    _ => Self::Other {
                        raw_event: event,
                        raw_data: EventRawData(param),
                    },
                }
            }
        }
    }

    pub struct EspAvrcc<'d, M, T>
    where
        M: BtClassicEnabled,
        T: Borrow<BtDriver<'d, M>>,
    {
        _driver: T,
        initialized: AtomicBool,
        _p: PhantomData<&'d ()>,
        _m: PhantomData<M>,
    }

    impl<'d, M, T> EspAvrcc<'d, M, T>
    where
        M: BtClassicEnabled,
        T: Borrow<BtDriver<'d, M>>,
    {
        pub const fn new(driver: T) -> Result<Self, EspError> {
            Ok(Self {
                _driver: driver,
                initialized: AtomicBool::new(false),
                _p: PhantomData,
                _m: PhantomData,
            })
        }

        pub fn initialize<F>(&self, events_cb: F) -> Result<(), EspError>
        where
            F: Fn(AvrccEvent) + Send + 'd,
        {
            CALLBACK.set(events_cb)?;

            esp!(unsafe { esp_avrc_ct_init() })?;
            esp!(unsafe { esp_avrc_ct_register_callback(Some(Self::event_handler)) })?;

            self.initialized.store(true, Ordering::SeqCst);

            Ok(())
        }

        pub fn set_player_settings(
            &self,
            transaction_label: u8,
            attribute: PlayerAttributeId,
        ) -> Result<(), EspError> {
            let (attribute_id, attribute_value) = attribute.into();
            esp!(unsafe {
                esp_avrc_ct_send_set_player_value_cmd(
                    transaction_label,
                    attribute_id as _,
                    attribute_value,
                )
            })
        }

        pub fn request_capabilities(&self, transaction_label: u8) -> Result<(), EspError> {
            esp!(unsafe { esp_avrc_ct_send_get_rn_capabilities_cmd(transaction_label) })
        }

        pub fn register_notification(
            &self,
            transaction_label: u8,
            notification: NotificationType,
        ) -> Result<(), EspError> {
            esp!(unsafe {
                esp_avrc_ct_send_register_notification_cmd(
                    transaction_label,
                    notification as _,
                    0, /*TODO*/
                )
            })
        }

        pub fn set_volume(&self, transaction_label: u8, volume: u8) -> Result<(), EspError> {
            esp!(unsafe { esp_avrc_ct_send_set_absolute_volume_cmd(transaction_label, volume) })
        }

        // TODO
        // pub fn set_metadata(&self, transaction_label: u8) -> Result<(), EspError> {
        //     esp!(unsafe { esp_avrc_ct_send_metadata_cmd(transaction_label) })
        // }

        pub fn send_passthrough(
            &self,
            transaction_label: u8,
            key_code: KeyCode,
            pressed: bool,
        ) -> Result<(), EspError> {
            esp!(unsafe {
                esp_avrc_ct_send_passthrough_cmd(
                    transaction_label,
                    key_code as _,
                    if pressed { 0 } else { 1 },
                )
            })
        }

        unsafe extern "C" fn event_handler(
            event: esp_avrc_ct_cb_event_t,
            param: *mut esp_avrc_ct_cb_param_t,
        ) {
            if let Some(param) = unsafe { param.as_ref() } {
                let event = AvrccEvent::from((event, param));

                info!("Got event {{ {:#?} }}", event);

                CALLBACK.call(event);
            }
        }
    }

    impl<'d, M, T> Drop for EspAvrcc<'d, M, T>
    where
        M: BtClassicEnabled,
        T: Borrow<BtDriver<'d, M>>,
    {
        fn drop(&mut self) {
            if self.initialized.load(Ordering::SeqCst) {
                esp!(unsafe { esp_avrc_ct_register_callback(None) }).unwrap();
                esp!(unsafe { esp_avrc_ct_deinit() }).unwrap();

                CALLBACK.clear().unwrap();
            }
        }
    }

    static CALLBACK: BtCallback<AvrccEvent, ()> = BtCallback::new(());
}
