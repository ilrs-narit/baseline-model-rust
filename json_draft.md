Observation
{
    "Header": {
        "Packet_Sync_Code":,
        "Packet_ID":,
        "Pakcet_Seq":,
        "Packet_Data_Len":,
        "Time":,
        "Data_Type":,
    }
    "Event_Group": [{
        "Particle_Time":
        "L1_XY_Position",
        "L1_X_Pulse_Height":,
        "L1_Y_Pulse_Height":,
        "L2_XY_Position":,
        "L2_X_Pulse_Height":,
        "L2_Y_Pulse_Height":,
        "L6_XY_Position":,
        "L6_X_Pulse_Height":,
        "L6_Y_Pulse_Height",
        "L7_XY_Position":,
        "L7_X_Pulse_Height":,
        "L7_Y_Pulse_Height":,
        "Galactic_Electron_Count":,
        "Albedo_Electron_Count":,
        "Galactic_Ion_Count":,
        "Albedo_Ion_Count":,
        "L1_Ion_Threshold":,
        "L1_Electron_Threshold":,
        "L2_Ion_Particle":,
        "L2_Electron_Particle":,
        "L3_Ion_Particle":,
        "L3_Electron_Particle":,
        "L4_Ion_Particle":,
        "L4_Electron_Particle":,
        "L5_Ion_Particle":,
        "L5_Electron_Particle":,
        "L6_Ion_Particle":,
        "L6_Electron_Particle":,
        "L7_Ion_Particle":,
        "L7_Electron_Particle":,
    }],
    "Tail": {
        "DSSD1_Temperature":,
        "FEE1_Current":,
        "FEE1_Temperature":,
        "DSSD7_Temperature":,
        "FEE2_Current":,
        "FEE2_Temperature":,
        "FEE1_Threshold":,
        "FEE2_Threshold":,
        "BGO1_Bias_Voltage":,
        "BGO2_Bias_Voltage":,
        "BGO3_Bias_Voltage":,
        "BGO2_Temperature":,
    }
    "Padding":,
    "Checksum":,
}


Calibration
{
    "Header": {
        "Packet_Sync_Code":,
        "Packet_ID":,
        "Pakcet_Seq":,
        "Packet_Data_Len":,
        "Time":,
        "Data_Type":,
        "Sample_Index":,
    },
    "Calibration_Voltage_Step": {
        "00V": {
            "L1":,
            "L2":,
            "L6":,
            "L7":, 
        }
        "01V": {
            "L1":,
            "L2":,
            "L6":,
            "L7":, 
        }
        "02V": {
            "L1":,
            "L2":,
            "L6":,
            "L7":, 
        }
        "03V": {
            "L1":,
            "L2":,
            "L6":,
            "L7":, 
        }
        "04V": {
            "L1":,
            "L2":,
            "L6":,
            "L7":, 
        }
        "05V": {
            "L1":,
            "L2":,
            "L6":,
            "L7":, 
        }
        "06V": {
            "L1":,
            "L2":,
            "L6":,
            "L7":, 
        }
        "07V": {
            "L1":,
            "L2":,
            "L6":,
            "L7":, 
        }
        "08V": {
            "L1":,
            "L2":,
            "L6":,
            "L7":, 
        }
        "09V": {
            "L1":,
            "L2":,
            "L6":,
            "L7":, 
        }
        "10V": {
            "L1":,
            "L2":,
            "L6":,
            "L7":, 
        }
    },
    "Tail": {
        "DSSD1_Temperature":,
        FEE1_Current":,
        FEE1_Temperature":,
        DSSD7_Temperature":,
        FEE2_Current":,
        FEE2_Temperature":,
        FEE1_Threshold":,
        FEE2_Threshold":,
        DSSD1_Temperature_dup":,
        DSSD4_Temperature":,
    },
        Reserved":,
        Checksum":,
}

Flux
{
    "Header": {
        "Packet_Sync_Code":,
        "Packet_ID":,
        "Pakcet_Seq":,
        "Packet_Data_Len":,
        "Time":,
        "Data_Type":,
        "Particle_Time":,
    },
    "Information": {
        "Particle_Counts_L1":,
        "Particle Counts L2":,
        "Particle Counts L3":,
        "Particle Counts L4":,
        "Particle Counts L5":,
        "Particle Counts L6":,
        "Particle Counts L7":,
        "Particle Info":,
    }
    "Tail": {
        "DSSD 1 Temperature":,
        "FEE1 Current":,
        "FEE1 Temperature":,
        "DSSD 7 Temperature":,
        "FEE2 Current":,
        "FEE2 Temperature":,
        "FEE1 Threshold":,
        "FEE2 Threshold":,
        "BGO1 Bias Voltage":,
        "BGO2 Bias Voltage":,
        "BGO3 Bias Voltage":,
        "BGO2 Temperature":,
    },
        "Reserved":,
        "Checksum":,
}

Baseline
{
    "Header": {
        "Packet_Sync_Code":,
        "Packet_ID":,
        "Pakcet_Seq":,
        "Packet_Data_Len":,
        "Time":,
        "Data_Type":,
        "Sample_Index":,
    },
    "Data_Acquisition":, {
        "L1": {
            "block_1":,
            "block_2":,
            "block_3":,
            "block_4":,
            "block_5":,
            "block_6":,
            "block_7":,
            "block_8":,
            "block_9":,
            "block_10":,
            "block_11":,
            "block_12":,
            "block_13":,
            "block_14":,
            "block_15":,
        },
        "L2": {
            "block_1":,
            "block_2":,
            "block_3":,
            "block_4":,
            "block_5":,
            "block_6":,
            "block_7":,
            "block_8":,
            "block_9":,
            "block_10":,
            "block_11":,
            "block_12":,
            "block_13":,
            "block_14":,
            "block_15":,
        },
        "L6": {
            "block_1":,
            "block_2":,
            "block_3":,
            "block_4":,
            "block_5":,
            "block_6":,
            "block_7":,
            "block_8":,
            "block_9":,
            "block_10":,
            "block_11":,
            "block_12":,
            "block_13":,
            "block_14":,
            "block_15":,
        },
        "L7": {
            "block_1":,
            "block_2":,
            "block_3":,
            "block_4":,
            "block_5":,
            "block_6":,
            "block_7":,
            "block_8":,
            "block_9":,
            "block_10":,
            "block_11":,
            "block_12":,
            "block_13":,
            "block_14":,
            "block_15":,
        },
    },
    "Tail": {
        "DSSD1_Temperature":,
        "FEE1_Current":,
        "FEE1_Temperature":,
        "DSSD7_Temperature":,
        "FEE2_Current":,
        "FEE2_Temperature":,
        "FEE1_Threshold":,
        "FEE2_Threshold":,
        "DSSD1_Temperature_dup":,
        "DSSD4_Temperature":,
    },
    "Reserved":,
    "Checksum":,
}