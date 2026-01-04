#[cfg(feature = "server")]
mod logging_tests {
    use std::io::Write;
    use std::sync::{Arc, Mutex};

    use barforge_web::server::auth::resolve_redirect_target;
    use tracing_subscriber::fmt::{MakeWriter, format::FmtSpan};

    #[derive(Clone, Default)]
    struct TestWriter {
        buffer: Arc<Mutex<Vec<u8>>>,
    }

    impl TestWriter {
        fn output(&self) -> String {
            let buffer = self.buffer.lock().expect("lock buffer");
            String::from_utf8_lossy(&buffer).to_string()
        }
    }

    impl Write for TestWriter {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            let mut buffer = self.buffer.lock().expect("lock buffer");
            buffer.extend_from_slice(buf);
            Ok(buf.len())
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    impl<'a> MakeWriter<'a> for TestWriter {
        type Writer = TestWriter;

        fn make_writer(&'a self) -> Self::Writer {
            self.clone()
        }
    }

    #[test]
    fn resolve_redirect_target_emits_span() {
        let writer = TestWriter::default();
        let subscriber = tracing_subscriber::fmt()
            .with_span_events(FmtSpan::ENTER)
            .with_writer(writer.clone())
            .finish();

        tracing::subscriber::with_default(subscriber, || {
            let _ = resolve_redirect_target(Some("/dashboard"));
        });

        let output = writer.output();
        assert!(output.contains("resolve_redirect_target"));
    }
}
