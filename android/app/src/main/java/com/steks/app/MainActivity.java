package com.steks.app;

import com.getcapacitor.BridgeActivity;

public class MainActivity extends BridgeActivity {

    @Override
    public void onCreate(Bundle savedInstanceState) {
        registerPlugin(CapacitorGameConnectPlugin.class);
        super.onCreate(savedInstanceState);
    }

}
