use resend_rs::Resend;

#[derive(Clone)]
pub struct ResendClient {
    pub resend: Resend,
}

impl ResendClient {
    pub fn new() -> Self {
        let resend = Resend::default();
        ResendClient { resend }
    }
}

pub fn verify_email_template(name: &str, verify_link: &str) -> String {
    format!(
        r#"
<!DOCTYPE html PUBLIC "-//W3C//DTD XHTML 1.0 Transitional//EN" 
  "http://www.w3.org/TR/xhtml1/DTD/xhtml1-transitional.dtd">
<html dir="ltr" lang="en">
  <head>
    <meta content="text/html; charset=UTF-8" http-equiv="Content-Type" />
    <meta name="x-apple-disable-message-reformatting" />
  </head>
  <body style="background-color:#f6f9fc">
    <table
      border="0"
      width="100%"
      cellpadding="0"
      cellspacing="0"
      role="presentation"
      align="center">
      <tbody>
        <tr>
          <td style="background-color:#f6f9fc;padding:10px 0">
            <div
              style="display:none;overflow:hidden;line-height:1px;opacity:0;max-height:0;max-width:0"
              data-skip-in-text="true">
              Verify your email for Axum-Rest
            </div>
            <table
              align="center"
              width="100%"
              border="0"
              cellpadding="0"
              cellspacing="0"
              role="presentation"
              style="max-width:37.5em;background-color:#ffffff;border:1px solid #f0f0f0;padding:45px">
              <tbody>
                <tr style="width:100%">
                  <td>
                    <table
                      align="center"
                      width="100%"
                      border="0"
                      cellpadding="0"
                      cellspacing="0"
                      role="presentation">
                      <tbody>
                        <tr>
                          <td>
                            <p
                              style="font-size:16px;line-height:26px;
                              font-family:'Open Sans','HelveticaNeue-Light',
                              'Helvetica Neue Light','Helvetica Neue',
                              Helvetica,Arial,'Lucida Grande',sans-serif;
                              font-weight:300;color:#404040;
                              margin-top:16px;margin-bottom:16px">
                              Hi {},
                            </p>
                            <p
                              style="font-size:16px;line-height:26px;
                              font-family:'Open Sans','HelveticaNeue-Light',
                              'Helvetica Neue Light','Helvetica Neue',
                              Helvetica,Arial,'Lucida Grande',sans-serif;
                              font-weight:300;color:#404040;
                              margin-top:16px;margin-bottom:16px">
                              Thanks for signing up to <b>Axum-Rest</b>!  
                              Please confirm your email address by clicking the button below:
                            </p>
                            <a
                              href="{}"
                              style="line-height:100%;text-decoration:none;display:block;
                              max-width:100%;background-color:#2563eb;border-radius:4px;
                              color:#fff;font-family:'Open Sans','Helvetica Neue',Arial;
                              font-size:15px;text-align:center;width:210px;
                              padding:14px 7px"
                              target="_blank">
                              <span style="display:inline-block;line-height:120%;
                                mso-padding-alt:0px;mso-text-raise:10.5px">
                                Verify Email
                              </span>
                            </a>
                            <p
                              style="font-size:14px;line-height:22px;
                              font-family:'Open Sans','HelveticaNeue-Light',
                              'Helvetica Neue Light','Helvetica Neue',
                              Helvetica,Arial,'Lucida Grande',sans-serif;
                              font-weight:300;color:#6b7280;
                              margin-top:12px;margin-bottom:16px">
                              ️This verification link will expire in 15 minutes.
                            </p>
                            <p
                              style="font-size:16px;line-height:26px;
                              font-family:'Open Sans','HelveticaNeue-Light',
                              'Helvetica Neue Light','Helvetica Neue',
                              Helvetica,Arial,'Lucida Grande',sans-serif;
                              font-weight:300;color:#404040;
                              margin-top:16px;margin-bottom:16px">
                              If you didn’t create an account, you can safely ignore this message.
                            </p>
                            <p
                              style="font-size:16px;line-height:26px;
                              font-family:'Open Sans','HelveticaNeue-Light',
                              'Helvetica Neue Light','Helvetica Neue',
                              Helvetica,Arial,'Lucida Grande',sans-serif;
                              font-weight:300;color:#404040;
                              margin-top:16px;margin-bottom:16px">
                              Cheers,<br />
                              The Axum-Rest Team
                            </p>
                          </td>
                        </tr>
                      </tbody>
                    </table>
                  </td>
                </tr>
              </tbody>
            </table>
          </td>
        </tr>
      </tbody>
    </table>
  </body>
</html>

    "#,
        name, verify_link
    )
}

pub fn reset_password_template(name: &str, reset_link: &str) -> String {
    format!(
        r#"
      <!DOCTYPE html PUBLIC "-//W3C//DTD XHTML 1.0 Transitional//EN" 
  "http://www.w3.org/TR/xhtml1/DTD/xhtml1-transitional.dtd">
<html dir="ltr" lang="en">
  <head>
    <meta content="text/html; charset=UTF-8" http-equiv="Content-Type" />
    <meta name="x-apple-disable-message-reformatting" />
  </head>
  <body style="background-color:#f6f9fc">
    <table
      border="0"
      width="100%"
      cellpadding="0"
      cellspacing="0"
      role="presentation"
      align="center"
    >
      <tbody>
        <tr>
          <td style="background-color:#f6f9fc;padding:10px 0">
            <div
              style="display:none;overflow:hidden;line-height:1px;opacity:0;max-height:0;max-width:0"
              data-skip-in-text="true"
            >
              Reset your password for Axum-Rest
            </div>
            <table
              align="center"
              width="100%"
              border="0"
              cellpadding="0"
              cellspacing="0"
              role="presentation"
              style="max-width:37.5em;background-color:#ffffff;border:1px solid #f0f0f0;padding:45px"
            >
              <tbody>
                <tr style="width:100%">
                  <td>
                    <table
                      align="center"
                      width="100%"
                      border="0"
                      cellpadding="0"
                      cellspacing="0"
                      role="presentation"
                    >
                      <tbody>
                        <tr>
                          <td>
                            <p
                              style="font-size:16px;line-height:26px;
                              font-family:'Open Sans','HelveticaNeue-Light',
                              'Helvetica Neue Light','Helvetica Neue',
                              Helvetica,Arial,'Lucida Grande',sans-serif;
                              font-weight:300;color:#404040;
                              margin-top:16px;margin-bottom:16px"
                            >
                              Hi {},
                            </p>
                            <p
                              style="font-size:16px;line-height:26px;
                              font-family:'Open Sans','HelveticaNeue-Light',
                              'Helvetica Neue Light','Helvetica Neue',
                              Helvetica,Arial,'Lucida Grande',sans-serif;
                              font-weight:300;color:#404040;
                              margin-top:16px;margin-bottom:16px"
                            >
                              We received a request to reset your password for
                              <b>Axum-Rest</b>. Click the button below to set a
                              new password:
                            </p>
                            <a
                              href="{}"
                              style="line-height:100%;text-decoration:none;display:block;
                              max-width:100%;background-color:#2563eb;border-radius:4px;
                              color:#fff;font-family:'Open Sans','Helvetica Neue',Arial;
                              font-size:15px;text-align:center;width:210px;
                              padding:14px 7px"
                              target="_blank"
                            >
                              <span
                                style="display:inline-block;line-height:120%;
                                mso-padding-alt:0px;mso-text-raise:10.5px"
                              >
                                Reset Password
                              </span>
                            </a>
                            <p
                              style="font-size:14px;line-height:22px;
                              font-family:'Open Sans','HelveticaNeue-Light',
                              'Helvetica Neue Light','Helvetica Neue',
                              Helvetica,Arial,'Lucida Grande',sans-serif;
                              font-weight:300;color:#6b7280;
                              margin-top:12px;margin-bottom:16px"
                            >
                              This link will expire in 30 minutes for security reasons.
                            </p>
                            <p
                              style="font-size:16px;line-height:26px;
                              font-family:'Open Sans','HelveticaNeue-Light',
                              'Helvetica Neue Light','Helvetica Neue',
                              Helvetica,Arial,'Lucida Grande',sans-serif;
                              font-weight:300;color:#404040;
                              margin-top:16px;margin-bottom:16px"
                            >
                              If you didn’t request a password reset, you can safely
                              ignore this message. Your password will remain unchanged.
                            </p>
                            <p
                              style="font-size:16px;line-height:26px;
                              font-family:'Open Sans','HelveticaNeue-Light',
                              'Helvetica Neue Light','Helvetica Neue',
                              Helvetica,Arial,'Lucida Grande',sans-serif;
                              font-weight:300;color:#404040;
                              margin-top:16px;margin-bottom:16px"
                            >
                              Cheers,<br />
                              The Axum-Rest Team
                            </p>
                          </td>
                        </tr>
                      </tbody>
                    </table>
                  </td>
                </tr>
              </tbody>
            </table>
          </td>
        </tr>
      </tbody>
    </table>
  </body>
</html>

    "#,
        name, reset_link
    )
}
