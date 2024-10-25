package io.p01def.disposables.protocol;

import java.util.ArrayList;
import java.util.List;

import com.fasterxml.jackson.annotation.JsonProperty;

public class V1SetupMessage {
    public int port = 4;
	@JsonProperty("wait_for")
    public List<V1WaitCondition> waitFor = new ArrayList<>();
	@JsonProperty("ready_timeout_s")
	public long readyTimeoutS = 15;
    public List<String[]> files = new ArrayList<>();
}

