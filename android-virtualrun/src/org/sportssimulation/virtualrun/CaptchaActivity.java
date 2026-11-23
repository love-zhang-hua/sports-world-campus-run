package org.sportssimulation.virtualrun;
import android.app.*;import android.content.*;import android.os.*;import android.webkit.*;import android.net.Uri;import java.io.*;import org.json.*;
/** Isolated interactive provider challenge; no account credentials or account bridge. */
public final class CaptchaActivity extends Activity {
 private WebView web;
 @Override public void onCreate(Bundle b){super.onCreate(b);web=new WebView(this);setContentView(web);web.getSettings().setJavaScriptEnabled(true);web.getSettings().setDomStorageEnabled(true);web.getSettings().setAllowFileAccess(false);web.getSettings().setAllowContentAccess(false);web.getSettings().setMixedContentMode(WebSettings.MIXED_CONTENT_NEVER_ALLOW);web.addJavascriptInterface(new Bridge(),"HumanCaptcha");web.setWebViewClient(new WebViewClient(){@Override public boolean shouldOverrideUrlLoading(WebView w,WebResourceRequest r){return true;}@Override public WebResourceResponse shouldInterceptRequest(WebView w,WebResourceRequest r){Uri u=r.getUrl();try{if("https".equals(u.getScheme())&&"captcha.local".equals(u.getHost())&&("/index.html".equals(u.getPath())||"/captcha.js".equals(u.getPath())))return new WebResourceResponse(u.getPath().endsWith(".js")?"application/javascript":"text/html","UTF-8",getAssets().open("captcha"+u.getPath()));if("https".equals(u.getScheme()))return null;}catch(Exception ignored){}return new WebResourceResponse("text/plain","UTF-8",new ByteArrayInputStream(new byte[0]));}});web.loadUrl("https://captcha.local/index.html");}
 public final class Bridge {
  @JavascriptInterface public void cancel(){runOnUiThread(()->finish());}
  @JavascriptInterface public void complete(String raw){try{JSONObject p=new JSONObject(raw);for(String k:new String[]{"lot_number","captcha_output","pass_token","gen_time"}){String v=p.getString(k);if(v.isEmpty()||v.length()>16384)throw new IllegalArgumentException();}runOnUiThread(()->{setResult(RESULT_OK,new Intent().putExtra("proof",raw));finish();});}catch(Exception e){runOnUiThread(()->android.widget.Toast.makeText(CaptchaActivity.this,"验证结果不完整，请重新验证",1).show());}}
 }
 @Override protected void onDestroy(){web.removeJavascriptInterface("HumanCaptcha");web.destroy();super.onDestroy();}
}
