package com.steksgame.app;

import com.getcapacitor.BridgeActivity;
import com.openforge.capacitorgameconnect.CapacitorGameConnectPlugin;

import android.os.Bundle;

public class MainActivity extends BridgeActivity {

    @Override
    public void onCreate(Bundle savedInstanceState) {
        registerPlugin(CapacitorGameConnectPlugin.class);
        super.onCreate(savedInstanceState);
    }
}
